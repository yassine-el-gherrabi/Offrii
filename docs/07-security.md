# Securite & RGPD

Ce document detaille les mesures de securite implementees dans Offrii ainsi que la conformite au Reglement General sur la Protection des Donnees (RGPD). Chaque affirmation est etayee par une reference au code source ou a la configuration correspondante.

---

## 1. Authentification

### 1.1 Architecture JWT RS256 asymetrique

Offrii utilise des JSON Web Tokens signes avec l'algorithme **RS256** (RSA-SHA256 asymetrique). La cle privee signe les tokens cote backend ; seule la cle publique est necessaire pour la verification. Ce choix permet de valider des tokens dans des services tiers sans exposer la cle de signature.

| Parametre | Valeur | Fichier source |
|---|---|---|
| Algorithme | RS256 (RSA 2048 bits) | `utils/jwt.rs:14-18` |
| Access token TTL | 15 minutes | `utils/jwt.rs:8-9` |
| Refresh token TTL | 7 jours | `utils/jwt.rs:11-12` |
| Issuer | `offrii-api` | `utils/jwt.rs:26` |
| Audience | `offrii-app` | `utils/jwt.rs:29` |

**Structure des claims JWT** :

```
{
  "sub": "<user_id UUID>",
  "exp": <timestamp>,
  "iat": <timestamp>,
  "jti": "<UUID v4 unique>",
  "token_type": "access" | "refresh",
  "iss": "offrii-api",
  "aud": "offrii-app",
  "token_version": <int>,
  "is_admin": <bool>
}
```

**Securites supplementaires** :

- **Validation du type** : un refresh token ne peut pas etre utilise comme access token et inversement (`jwt.rs:182-187`).
- **Validation iss/aud** : les claims `issuer` et `audience` sont verifies a chaque validation (`jwt.rs:174-176`).
- **Clef par environnement** : en build release, les variables `JWT_PRIVATE_KEY_FILE` et `JWT_PUBLIC_KEY_FILE` sont obligatoires. La generation ephemere est interdite (`jwt.rs:127-129`).

### 1.2 Rotation atomique des refresh tokens

Le mecanisme de refresh utilise une **rotation atomique** dans une transaction PostgreSQL :

1. Le client envoie son refresh token actuel.
2. Le backend valide le token RS256 et verifie le `token_version` contre la base.
3. Dans une **transaction SQL** :
   - L'ancien refresh token est revoque via `UPDATE` (verrouillage de ligne pour empecher les refresh concurrents).
   - Un nouveau couple access + refresh est genere et le nouveau refresh hash est insere.
4. Si l'ancien token n'existe plus (replay), la requete echoue avec `401`.

**Reference** : `auth_service.rs:387-414`

Le nombre maximum de refresh tokens actifs par utilisateur est plafonne a **5** (`auth_service.rs:29`). Au-dela, les tokens les plus anciens sont revoques automatiquement.

### 1.3 Revocation de tokens

Offrii dispose de deux mecanismes de revocation complementaires :

**Revocation unitaire (JTI blacklist Redis)** :

Lors du logout, le `jti` (identifiant unique) de l'access token est inscrit dans Redis avec un TTL egal au temps restant avant expiration du token. A chaque requete authentifiee, le middleware verifie l'absence du JTI dans la blacklist.

```
Redis key : blacklist:<jti>
TTL       : remaining seconds until token expiration
```

**Reference** : `auth_service.rs:427-447`, `auth_extractor.rs:52-69`

**Revocation de masse (token_version)** :

Chaque utilisateur possede un champ `token_version` en base. L'incrementer invalide instantanement **tous** les tokens existants de cet utilisateur, puisque le middleware compare la version du token avec la version en base (via un cache Redis `tkver:<user_id>`).

```
Redis key : tkver:<user_id>
Verification : claims.token_version < cached_version → 401
```

**Reference** : `auth_extractor.rs:70-74`

**Politique fail-closed** : si Redis est indisponible lors de la verification, la requete est rejetee avec une erreur 500 (pas de degradation silencieuse). Reference : `auth_extractor.rs:42-49`.

### 1.4 Middleware d'authentification

Le middleware Axum `AuthUser` est un extracteur (`FromRequestParts`) qui :

1. Extrait le header `Authorization: Bearer <token>` (case-insensitive sur le scheme).
2. Valide la signature RS256, l'expiration, l'issuer et l'audience.
3. Verifie la blacklist JTI **et** le `token_version` dans Redis via un **pipeline** (une seule roundtrip).
4. Retourne un `AuthUser { user_id, jti, exp, is_admin }` injecte dans les handlers.

Un extracteur `AdminUser` verifie en plus le claim `is_admin` et rejette avec `403 Forbidden` si l'utilisateur n'est pas administrateur.

**Reference** : `middleware/auth_extractor.rs`

---

## 2. Hashage des mots de passe

### 2.1 Argon2id avec parametres OWASP 2026

| Parametre | Valeur | Recommandation OWASP |
|---|---|---|
| Algorithme | Argon2id v0x13 | Argon2id |
| Memoire (m) | 19 456 KiB (~19 Mo) | >= 19 MiB |
| Iterations (t) | 2 | >= 2 |
| Parallelisme (p) | 1 | 1 |

Le hashage est execute sur un **blocking thread** (`tokio::task::spawn_blocking`) pour ne pas bloquer le runtime async.

**Reference** : `utils/hash.rs:7-11`

### 2.2 Protection anti-timing (DUMMY_HASH)

Lors d'une tentative de connexion avec un email inexistant, le backend execute **quand meme** la verification Argon2id contre un hash factice pre-calcule (`DUMMY_HASH`). Cela rend le temps de reponse **indistinguable** d'une tentative avec un mauvais mot de passe, empechant un attaquant de determiner si un email est inscrit via une attaque par canal auxiliaire (timing side-channel).

```rust
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash::hash_password("timing-safe-dummy").expect("failed to generate dummy hash")
});
```

**Reference** : `auth_service.rs:34-37`, `auth_service.rs:306-314`

---

## 3. Politique de mots de passe

La politique de mots de passe suit les recommandations **OWASP 2024+** et est verifiee a l'inscription.

| Regle | Implementation | Reference |
|---|---|---|
| Longueur minimale | 8 caracteres (comptage Unicode, pas octets) | `password_policy.rs:19` |
| Longueur maximale | 128 caracteres | `password_policy.rs:22` |
| Mots de passe courants | Base de 10 000 mots de passe les plus repandus | `common_passwords.rs` |
| Fuites de donnees (HIBP) | API Have I Been Pwned (k-Anonymity) | `hibp.rs` |

### 3.1 Verification HIBP (Have I Been Pwned)

Le mot de passe est hashe en SHA-1, et seuls les **5 premiers caracteres** du hash sont envoyes a l'API HIBP (protocole k-Anonymity). La comparaison du suffixe se fait localement. Le mot de passe complet n'est jamais transmis.

**Politique fail-open** : si l'API HIBP est injoignable (timeout 5s), le mot de passe est accepte avec un avertissement en logs. Ce choix evite de bloquer les inscriptions en cas de panne d'un service tiers.

**Reference** : `utils/hibp.rs:19-56`

### 3.2 Ordre de verification

Les verifications sont executees dans un ordre optimise (du moins couteux au plus couteux) :

1. **Longueur** (O(n), instantane)
2. **Mots de passe courants** (lookup local, O(1) via HashSet)
3. **HIBP** (appel reseau, le plus lent)

Si l'une des etapes echoue, les suivantes ne sont pas executees.

---

## 4. Protection API -- OWASP Top 10

| # | Risque OWASP 2021 | Mesure implementee | Reference |
|---|---|---|---|
| A01 | **Broken Access Control** | Chaque requete SQL filtre par `auth_user.user_id` extrait du JWT. Un utilisateur ne peut jamais acceder aux ressources d'un autre. | `handlers/items.rs` (tous les handlers) |
| A02 | **Cryptographic Failures** | JWT RS256, Argon2id, TLS via Caddy (HSTS 2 ans preload), refresh tokens hashes en SHA-256 avant stockage. | `utils/jwt.rs`, `utils/hash.rs`, `utils/token_hash.rs` |
| A03 | **Injection** | Toutes les requetes SQL utilisent **SQLx avec requetes parametrees** (`$1`, `$2`). Aucune concatenation de chaines dans les requetes. | Toutes les couches `repositories/` |
| A04 | **Insecure Design** | Architecture en couches (handlers -> services -> repositories via traits). Validation en entree (`validator`), types forts (UUID, enum). | `dto/`, `models/`, `handlers/` |
| A05 | **Security Misconfiguration** | CSP headers via Caddy, CORS configure, Swagger protege par Basic Auth en production, `/metrics` restreint. | `Caddyfile.prod` |
| A06 | **Vulnerable Components** | `cargo audit` en CI, scan Trivy sur les images Docker, dependances a jour. | `.github/workflows/security.yml` |
| A07 | **Auth Failures** | Rate limiting login (10/5min), Argon2id, JTI blacklist, token_version, max 5 refresh tokens. | `auth_service.rs`, `auth_extractor.rs` |
| A08 | **Software & Data Integrity** | CI/CD GitHub Actions avec tests obligatoires, images Docker signees, deploiement via SSH protege. | `.github/workflows/` |
| A09 | **Security Logging** | Logs structures (tracing + Loki), connection logs (IP + user-agent), metriques Prometheus. | `auth_service.rs:103-112`, monitoring stack |
| A10 | **SSRF** | Blocage des IPs privees, loopback, link-local, CGNAT et broadcast dans le fetcher OG metadata. | `og_service.rs:12-24` |

### 4.1 Protection IDOR (Insecure Direct Object Reference)

Chaque endpoint qui manipule des ressources utilisateur recoit un `AuthUser` extrait du JWT et passe le `user_id` comme parametre de filtrage a la couche service/repository. Exemple :

```rust
// handlers/items.rs — le user_id vient du JWT, jamais du client
async fn list_items(..., auth_user: AuthUser, ...) -> ... {
    state.items.list_items(auth_user.user_id, &query).await
}

async fn delete_item(..., auth_user: AuthUser, Path(id): Path<Uuid>) -> ... {
    state.items.delete_item(id, auth_user.user_id).await
}
```

Un utilisateur ne peut ni lire, ni modifier, ni supprimer les items d'un autre utilisateur. La verification se fait au niveau SQL (`WHERE user_id = $1`), pas en logique applicative.

---

## 5. Rate limiting

Offrii utilise un rate limiting **multi-couche** :

| Couche | Outil | Limite | Scope |
|---|---|---|---|
| Reverse proxy | Caddy | 100 requetes/minute | IP globale |
| Authentification | Caddy | 10 requetes/minute | Routes `/auth/*` |
| Login backend | Redis (`INCR` + `EXPIRE`) | 10 tentatives / 5 min | Par identifiant (email/username) |
| Metier | Redis (cooldowns) | Variable | Actions specifiques (ex: envoi email) |

**Fonctionnement du rate limit login** (`auth_service.rs:274-291`) :

1. Cle Redis : `login:attempts:<identifier>` (email ou username normalise en lowercase).
2. `INCR` atomique a chaque tentative.
3. `EXPIRE 300` (5 minutes) positionne une seule fois (premier appel).
4. Au-dela de 10 tentatives, retour immediat avec erreur.

---

## 6. Protection SSRF

Le service de recuperation de metadonnees OpenGraph (`og_service.rs`) est protege contre les attaques SSRF (Server-Side Request Forgery) :

### 6.1 Plages d'adresses bloquees

| Plage | Type | Raison |
|---|---|---|
| `127.0.0.0/8` | Loopback IPv4 | Acces au serveur local |
| `10.0.0.0/8` | RFC 1918 private | Reseau interne |
| `172.16.0.0/12` | RFC 1918 private | Reseau interne |
| `192.168.0.0/16` | RFC 1918 private | Reseau interne |
| `169.254.0.0/16` | Link-local | **Metadonnees cloud** (ex: AWS IMDSv1 `169.254.169.254`) |
| `100.64.0.0/10` | CGNAT (RFC 6598) | Infrastructure operateur |
| `255.255.255.255` | Broadcast | Diffusion reseau |
| `0.0.0.0` | Unspecified | Adresse non routable |
| `::1` | Loopback IPv6 | Acces au serveur local |
| `::` | Unspecified IPv6 | Adresse non routable |

### 6.2 Protections supplementaires

| Protection | Valeur |
|---|---|
| Schemas autorises | `http` et `https` uniquement |
| Timeout | 10 secondes |
| Taille maximale reponse | 1 Mo (double verification : Content-Length + taille reelle) |
| Redirections maximales | 5 |
| Resolution DNS | Toutes les adresses resolues sont verifiees avant la connexion |

**Reference** : `services/og_service.rs:12-51`

---

## 7. Conformite RGPD

### 7.1 Table de conformite

| Exigence RGPD | Article | Implementation | Reference |
|---|---|---|---|
| **Base legale : consentement** | Art. 6(1)(a) | Texte explicite sous le bouton d'inscription avec liens vers la Politique de confidentialite et les CGU. L'utilisateur doit s'inscrire pour consentir ; pas de case a cocher implicite. | Frontend iOS, ecran d'inscription |
| **Droit d'acces** | Art. 15 | Endpoint `GET /users/me/export` retourne un JSON complet de toutes les donnees de l'utilisateur (profil, items, cercles, amis, wishes, messages). | Backend API |
| **Droit a l'effacement** | Art. 17 | Endpoint `DELETE /users/me` supprime le compte et toutes les donnees associees en cascade (items, tokens, cercles, messages, logs). | Backend API |
| **Droit a la portabilite** | Art. 20 | Export JSON complet via le meme endpoint d'export, dans un format structure et lisible par machine. | Backend API |
| **Limitation de conservation** | Art. 5(1)(e) | Logs de connexion (IP + user-agent) conserves **12 mois** maximum, puis purges automatiquement. | `connection_logs` table + cron purge |
| **Inactivite des comptes** | Art. 5(1)(e) | Preavis par email a **23 mois** d'inactivite. Suppression automatique du compte a **24 mois**. Tracking via `last_active_at` (throttle 15 min via Redis). | `auth_extractor.rs:101-124`, cron inactivite |
| **Minimisation des donnees** | Art. 5(1)(c) | Seules les donnees necessaires au fonctionnement sont collectees. Pas de tracking analytique, pas de pixels tiers. | Architecture backend |
| **Securite du traitement** | Art. 32 | Argon2id, JWT RS256, TLS obligatoire, HSTS 2 ans, chiffrement at-rest (PostgreSQL), isolation Docker. | Sections 1-6 de ce document |
| **Notification de violation** | Art. 33-34 | Logs structures (Loki), alertes Grafana, monitoring en temps reel. Procedure de notification CNIL sous 72h. | Infrastructure monitoring |
| **Mediateur** | Art. 80 | Mediateur SMP designe, reference **#223702**. Mentionne dans les CGU et la Politique de confidentialite. | Documents legaux |

### 7.2 Suivi d'activite pour la gestion d'inactivite

Le champ `last_active_at` est mis a jour automatiquement par le middleware d'authentification, avec un **throttle de 15 minutes** via Redis pour eviter une ecriture SQL a chaque requete :

```rust
// auth_extractor.rs — mise a jour throttlee
let active_key = format!("active:{uid}");
redis::cmd("SET").arg(&active_key).arg("1")
    .arg("EX").arg(900)  // 15 min TTL
    .arg("NX")           // only if key doesn't exist
```

Si la cle Redis `active:<user_id>` n'existe pas, elle est creee (NX) avec un TTL de 900 secondes, et une mise a jour SQL est declenchee. Les requetes suivantes dans les 15 minutes ne generent aucune ecriture.

### 7.3 Logs de connexion

Chaque connexion reussie (login ou inscription) enregistre :

| Champ | Source |
|---|---|
| `user_id` | JWT `sub` claim |
| `ip` | Header `X-Forwarded-For` (Caddy) |
| `user_agent` | Header `User-Agent` |
| `created_at` | Timestamp automatique |

Duree de conservation : **12 mois** (purge automatique). Aucune autre donnee de navigation n'est collectee.

**Reference** : `auth_service.rs:103-112`

---

## 8. Documents legaux

Offrii dispose de trois documents legaux complets, disponibles en **francais et en anglais**, accessibles depuis l'application via des WebViews et heberges sur le serveur API.

| Document | Sections | Contenu principal | Conformite |
|---|---|---|---|
| **Mentions legales** | -- | Editeur (auto-entrepreneur), hebergeurs (Hetzner, Cloudflare, App Store), contact, directeur de publication. | LCEN (Loi pour la Confiance dans l'Economie Numerique) |
| **Politique de confidentialite** | 14 sections | Donnees collectees, base legale, droits des utilisateurs, durees de conservation, transferts internationaux, cookies, contact DPO. | RGPD art. 13-14 |
| **Conditions generales d'utilisation** | 15 sections | Objet, inscription, contenu utilisateur, propriete intellectuelle, responsabilites, signalement, moderation, sanctions, mediation. | DSA (Digital Services Act) |

### 8.1 Acces dans l'application

Les documents sont accessibles depuis l'ecran Profil de l'application iOS via trois liens dedies. Le composant `LegalView` charge les pages via une WebView pointant vers les endpoints de l'API :

- `/legal/mentions?lang=fr|en`
- `/legal/privacy?lang=fr|en`
- `/legal/terms?lang=fr|en`

La langue est detectee automatiquement en fonction des reglages de l'appareil.

**Reference** : `Features/Legal/LegalView.swift`

### 8.2 Points de conformite DSA (Digital Services Act)

Les CGU incluent les dispositions requises par le DSA :

- Mecanisme de signalement de contenu illicite
- Procedure de moderation transparente
- Sanctions progressives (avertissement, suspension, suppression)
- Information des utilisateurs sur les decisions de moderation
- Voies de recours (mediateur SMP, ref #223702)

---

## 9. Recapitulatif des references de securite

| Composant | Fichier(s) source | Description |
|---|---|---|
| JWT RS256 | `utils/jwt.rs` | Generation, validation, claims, key management |
| Hashage Argon2id | `utils/hash.rs` | Parametres OWASP, hash et verify |
| Politique mdp | `utils/password_policy.rs` | Longueur, mots courants, HIBP |
| HIBP k-Anonymity | `utils/hibp.rs` | Verification fuites via API |
| Mots de passe courants | `utils/common_passwords.rs` | Base locale 10k mots de passe |
| Auth middleware | `middleware/auth_extractor.rs` | Extraction JWT, blacklist, token_version |
| Auth service | `services/auth_service.rs` | Login, register, refresh, logout, DUMMY_HASH |
| Items handler | `handlers/items.rs` | Filtrage user_id sur chaque operation |
| OG metadata (SSRF) | `services/og_service.rs` | Blocage IPs privees, timeout, taille max |
| Legal views | `Features/Legal/LegalView.swift` | Acces documents legaux in-app |
| Prod checklist | `claudedocs/prod-checklist.md` | Audit de securite complet |

---

*Derniere mise a jour : mars 2026*

# Sécurité & RGPD

Ce document détaille les mesures de sécurité implémentées dans Offrii ainsi que la conformité au Règlement Général sur la Protection des Données (RGPD). Chaque affirmation est étayée par une référence au code source ou à la configuration correspondante.

---

## 1. Authentification

### 1.1 Architecture JWT RS256 asymétrique

Offrii utilise des JSON Web Tokens signés avec l'algorithme **RS256** (RSA-SHA256 asymétrique). La clé privée signe les tokens côté backend ; seule la clé publique est nécessaire pour la vérification. Ce choix permet de valider des tokens dans des services tiers sans exposer la clé de signature.

| Paramètre | Valeur | Fichier source |
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

**Sécurités supplémentaires** :

- **Validation du type** : un refresh token ne peut pas être utilisé comme access token et inversement (`jwt.rs:182-187`).
- **Validation iss/aud** : les claims `issuer` et `audience` sont vérifiés à chaque validation (`jwt.rs:174-176`).
- **Clef par environnement** : en build release, les variables `JWT_PRIVATE_KEY_FILE` et `JWT_PUBLIC_KEY_FILE` sont obligatoires. La génération éphémère est interdite (`jwt.rs:127-129`).

### 1.2 Rotation atomique des refresh tokens

Le mécanisme de refresh utilise une **rotation atomique** dans une transaction PostgreSQL :

1. Le client envoie son refresh token actuel.
2. Le backend valide le token RS256 et vérifie le `token_version` contre la base.
3. Dans une **transaction SQL** :
   - L'ancien refresh token est révoqué via `UPDATE` (verrouillage de ligne pour empêcher les refresh concurrents).
   - Un nouveau couple access + refresh est généré et le nouveau refresh hash est inséré.
4. Si l'ancien token n'existe plus (replay), la requête échoue avec `401`.

**Référence** : `auth_service.rs:387-414`

Le nombre maximum de refresh tokens actifs par utilisateur est plafonné à **5** (`auth_service.rs:29`). Au-delà, les tokens les plus anciens sont révoqués automatiquement.

### 1.3 Révocation de tokens

Offrii dispose de deux mécanismes de révocation complémentaires :

**Révocation unitaire (JTI blacklist Redis)** :

Lors du logout, le `jti` (identifiant unique) de l'access token est inscrit dans Redis avec un TTL égal au temps restant avant expiration du token. À chaque requête authentifiée, le middleware vérifie l'absence du JTI dans la blacklist.

```
Redis key : blacklist:<jti>
TTL       : remaining seconds until token expiration
```

**Référence** : `auth_service.rs:427-447`, `auth_extractor.rs:52-69`

**Révocation de masse (token_version)** :

Chaque utilisateur possède un champ `token_version` en base. L'incrémenter invalide instantanément **tous** les tokens existants de cet utilisateur, puisque le middleware compare la version du token avec la version en base (via un cache Redis `tkver:<user_id>`).

```
Redis key : tkver:<user_id>
Vérification : claims.token_version < cached_version → 401
```

**Référence** : `auth_extractor.rs:70-74`

**Politique fail-closed** : si Redis est indisponible lors de la vérification, la requête est rejetée avec une erreur 500 (pas de dégradation silencieuse). Référence : `auth_extractor.rs:42-49`.

### 1.4 Middleware d'authentification

Le middleware Axum `AuthUser` est un extracteur (`FromRequestParts`) qui :

1. Extrait le header `Authorization: Bearer <token>` (case-insensitive sur le scheme).
2. Valide la signature RS256, l'expiration, l'issuer et l'audience.
3. Vérifie la blacklist JTI **et** le `token_version` dans Redis via un **pipeline** (une seule roundtrip).
4. Retourne un `AuthUser { user_id, jti, exp, is_admin }` injecté dans les handlers.

Un extracteur `AdminUser` vérifie en plus le claim `is_admin` et rejette avec `403 Forbidden` si l'utilisateur n'est pas administrateur.

**Référence** : `middleware/auth_extractor.rs`

---

## 2. Hashage des mots de passe

### 2.1 Argon2id avec paramètres OWASP 2026

| Paramètre | Valeur | Recommandation OWASP |
|---|---|---|
| Algorithme | Argon2id v0x13 | Argon2id |
| Mémoire (m) | 19 456 KiB (~19 Mo) | >= 19 MiB |
| Itérations (t) | 2 | >= 2 |
| Parallélisme (p) | 1 | 1 |

Le hashage est exécuté sur un **blocking thread** (`tokio::task::spawn_blocking`) pour ne pas bloquer le runtime async.

**Référence** : `utils/hash.rs:7-11`

### 2.2 Protection anti-timing (DUMMY_HASH)

Lors d'une tentative de connexion avec un email inexistant, le backend exécute **quand même** la vérification Argon2id contre un hash factice pré-calculé (`DUMMY_HASH`). Cela rend le temps de réponse **indistinguable** d'une tentative avec un mauvais mot de passe, empêchant un attaquant de déterminer si un email est inscrit via une attaque par canal auxiliaire (timing side-channel).

```rust
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash::hash_password("timing-safe-dummy").expect("failed to generate dummy hash")
});
```

**Référence** : `auth_service.rs:34-37`, `auth_service.rs:306-314`

---

## 3. Politique de mots de passe

La politique de mots de passe suit les recommandations **OWASP 2024+** et est vérifiée à l'inscription.

| Règle | Implémentation | Référence |
|---|---|---|
| Longueur minimale | 8 caractères (comptage Unicode, pas octets) | `password_policy.rs:19` |
| Longueur maximale | 128 caractères | `password_policy.rs:22` |
| Mots de passe courants | Base de 10 000 mots de passe les plus répandus | `common_passwords.rs` |
| Fuites de données (HIBP) | API Have I Been Pwned (k-Anonymity) | `hibp.rs` |

### 3.1 Vérification HIBP (Have I Been Pwned)

Le mot de passe est hashé en SHA-1, et seuls les **5 premiers caractères** du hash sont envoyés à l'API HIBP (protocole k-Anonymity). La comparaison du suffixe se fait localement. Le mot de passe complet n'est jamais transmis.

**Politique fail-open** : si l'API HIBP est injoignable (timeout 5s), le mot de passe est accepté avec un avertissement en logs. Ce choix évite de bloquer les inscriptions en cas de panne d'un service tiers.

**Référence** : `utils/hibp.rs:19-56`

### 3.2 Ordre de vérification

Les vérifications sont exécutées dans un ordre optimisé (du moins coûteux au plus coûteux) :

1. **Longueur** (O(n), instantané)
2. **Mots de passe courants** (lookup local, O(1) via HashSet)
3. **HIBP** (appel réseau, le plus lent)

Si l'une des étapes échoue, les suivantes ne sont pas exécutées.

---

## 4. Protection API -- OWASP Top 10

| # | Risque OWASP 2021 | Mesure implémentée | Référence |
|---|---|---|---|
| A01 | **Broken Access Control** | Chaque requête SQL filtre par `auth_user.user_id` extrait du JWT. Un utilisateur ne peut jamais accéder aux ressources d'un autre. | `handlers/items.rs` (tous les handlers) |
| A02 | **Cryptographic Failures** | JWT RS256, Argon2id, TLS via Caddy (HSTS 2 ans preload), refresh tokens hashés en SHA-256 avant stockage. | `utils/jwt.rs`, `utils/hash.rs`, `utils/token_hash.rs` |
| A03 | **Injection** | Toutes les requêtes SQL utilisent **SQLx avec requêtes paramétrées** (`$1`, `$2`). Aucune concaténation de chaînes dans les requêtes. | Toutes les couches `repositories/` |
| A04 | **Insecure Design** | Architecture en couches (handlers -> services -> repositories via traits). Validation en entrée (`validator`), types forts (UUID, enum). | `dto/`, `models/`, `handlers/` |
| A05 | **Security Misconfiguration** | CSP headers via Caddy, CORS configuré, Swagger protégé par Basic Auth en production, `/metrics` restreint. | `Caddyfile.prod` |
| A06 | **Vulnerable Components** | `cargo audit` en CI, scan Trivy sur les images Docker, dépendances à jour. | `.github/workflows/security.yml` |
| A07 | **Auth Failures** | Rate limiting login (10/5min), Argon2id, JTI blacklist, token_version, max 5 refresh tokens. | `auth_service.rs`, `auth_extractor.rs` |
| A08 | **Software & Data Integrity** | CI/CD GitHub Actions avec tests obligatoires, images Docker signées, déploiement via SSH protégé. | `.github/workflows/` |
| A09 | **Security Logging** | Logs structurés (tracing + Loki), connection logs (IP + user-agent), métriques Prometheus. | `auth_service.rs:103-112`, monitoring stack |
| A10 | **SSRF** | Blocage des IPs privées, loopback, link-local, CGNAT et broadcast dans le fetcher OG metadata. | `og_service.rs:12-24` |

### 4.1 Protection IDOR (Insecure Direct Object Reference)

Chaque endpoint qui manipule des ressources utilisateur reçoit un `AuthUser` extrait du JWT et passe le `user_id` comme paramètre de filtrage à la couche service/repository. Exemple :

```rust
// handlers/items.rs — le user_id vient du JWT, jamais du client
async fn list_items(..., auth_user: AuthUser, ...) -> ... {
    state.items.list_items(auth_user.user_id, &query).await
}

async fn delete_item(..., auth_user: AuthUser, Path(id): Path<Uuid>) -> ... {
    state.items.delete_item(id, auth_user.user_id).await
}
```

Un utilisateur ne peut ni lire, ni modifier, ni supprimer les items d'un autre utilisateur. La vérification se fait au niveau SQL (`WHERE user_id = $1`), pas en logique applicative.

---

## 5. Rate limiting

Offrii utilise un rate limiting **multi-couche** :

| Couche | Outil | Limite | Scope |
|---|---|---|---|
| Reverse proxy | Caddy | 100 requêtes/minute | IP globale |
| Authentification | Caddy | 10 requêtes/minute | Routes `/auth/*` |
| Login backend | Redis (`INCR` + `EXPIRE`) | 10 tentatives / 5 min | Par identifiant (email/username) |
| Métier | Redis (cooldowns) | Variable | Actions spécifiques (ex: envoi email) |

**Fonctionnement du rate limit login** (`auth_service.rs:274-291`) :

1. Clé Redis : `login:attempts:<identifier>` (email ou username normalisé en lowercase).
2. `INCR` atomique à chaque tentative.
3. `EXPIRE 300` (5 minutes) positionné une seule fois (premier appel).
4. Au-delà de 10 tentatives, retour immédiat avec erreur.

---

## 6. Protection SSRF

Le service de récupération de métadonnées OpenGraph (`og_service.rs`) est protégé contre les attaques SSRF (Server-Side Request Forgery) :

### 6.1 Plages d'adresses bloquées

| Plage | Type | Raison |
|---|---|---|
| `127.0.0.0/8` | Loopback IPv4 | Accès au serveur local |
| `10.0.0.0/8` | RFC 1918 private | Réseau interne |
| `172.16.0.0/12` | RFC 1918 private | Réseau interne |
| `192.168.0.0/16` | RFC 1918 private | Réseau interne |
| `169.254.0.0/16` | Link-local | **Métadonnées cloud** (ex: AWS IMDSv1 `169.254.169.254`) |
| `100.64.0.0/10` | CGNAT (RFC 6598) | Infrastructure opérateur |
| `255.255.255.255` | Broadcast | Diffusion réseau |
| `0.0.0.0` | Unspecified | Adresse non routable |
| `::1` | Loopback IPv6 | Accès au serveur local |
| `::` | Unspecified IPv6 | Adresse non routable |

### 6.2 Protections supplémentaires

| Protection | Valeur |
|---|---|
| Schémas autorisés | `http` et `https` uniquement |
| Timeout | 10 secondes |
| Taille maximale réponse | 1 Mo (double vérification : Content-Length + taille réelle) |
| Redirections maximales | 5 |
| Résolution DNS | Toutes les adresses résolues sont vérifiées avant la connexion |

**Référence** : `services/og_service.rs:12-51`

---

## 7. Conformité RGPD

### 7.1 Table de conformité

| Exigence RGPD | Article | Implémentation | Référence |
|---|---|---|---|
| **Base légale : consentement** | Art. 6(1)(a) | Texte explicite sous le bouton d'inscription avec liens vers la Politique de confidentialité et les CGU. L'utilisateur doit s'inscrire pour consentir ; pas de case à cocher implicite. | Frontend iOS, écran d'inscription |
| **Droit d'accès** | Art. 15 | Endpoint `GET /users/me/export` retourne un JSON complet de toutes les données de l'utilisateur (profil, items, cercles, amis, wishes, messages). | Backend API |
| **Droit à l'effacement** | Art. 17 | Endpoint `DELETE /users/me` supprime le compte et toutes les données associées en cascade (items, tokens, cercles, messages, logs). | Backend API |
| **Droit à la portabilité** | Art. 20 | Export JSON complet via le même endpoint d'export, dans un format structuré et lisible par machine. | Backend API |
| **Limitation de conservation** | Art. 5(1)(e) | Logs de connexion (IP + user-agent) conservés **12 mois** maximum, puis purgés automatiquement. | `connection_logs` table + cron purge |
| **Inactivité des comptes** | Art. 5(1)(e) | Préavis par email à **23 mois** d'inactivité. Suppression automatique du compte à **24 mois**. Tracking via `last_active_at` (throttle 15 min via Redis). | `auth_extractor.rs:101-124`, cron inactivité |
| **Minimisation des données** | Art. 5(1)(c) | Seules les données nécessaires au fonctionnement sont collectées. Pas de tracking analytique, pas de pixels tiers. | Architecture backend |
| **Sécurité du traitement** | Art. 32 | Argon2id, JWT RS256, TLS obligatoire, HSTS 2 ans, chiffrement at-rest (PostgreSQL), isolation Docker. | Sections 1-6 de ce document |
| **Notification de violation** | Art. 33-34 | Logs structurés (Loki), alertes Grafana, monitoring en temps réel. Procédure de notification CNIL sous 72h. | Infrastructure monitoring |
| **Médiateur** | Art. 80 | Médiateur SMP désigné, référence **#223702**. Mentionné dans les CGU et la Politique de confidentialité. | Documents légaux |

### 7.2 Suivi d'activité pour la gestion d'inactivité

Le champ `last_active_at` est mis à jour automatiquement par le middleware d'authentification, avec un **throttle de 15 minutes** via Redis pour éviter une écriture SQL à chaque requête :

```rust
// auth_extractor.rs — mise à jour throttlée
let active_key = format!("active:{uid}");
redis::cmd("SET").arg(&active_key).arg("1")
    .arg("EX").arg(900)  // 15 min TTL
    .arg("NX")           // only if key doesn't exist
```

Si la clé Redis `active:<user_id>` n'existe pas, elle est créée (NX) avec un TTL de 900 secondes, et une mise à jour SQL est déclenchée. Les requêtes suivantes dans les 15 minutes ne génèrent aucune écriture.

### 7.3 Logs de connexion

Chaque connexion réussie (login ou inscription) enregistre :

| Champ | Source |
|---|---|
| `user_id` | JWT `sub` claim |
| `ip` | Header `X-Forwarded-For` (Caddy) |
| `user_agent` | Header `User-Agent` |
| `created_at` | Timestamp automatique |

Durée de conservation : **12 mois** (purge automatique). Aucune autre donnée de navigation n'est collectée.

**Référence** : `auth_service.rs:103-112`

---

## 8. Documents légaux

Offrii dispose de trois documents légaux complets, disponibles en **français et en anglais**, accessibles depuis l'application via des WebViews et hébergés sur le serveur API.

| Document | Sections | Contenu principal | Conformité |
|---|---|---|---|
| **Mentions légales** | -- | Éditeur (auto-entrepreneur), hébergeurs (Hetzner, Cloudflare, App Store), contact, directeur de publication. | LCEN (Loi pour la Confiance dans l'Économie Numérique) |
| **Politique de confidentialité** | 14 sections | Données collectées, base légale, droits des utilisateurs, durées de conservation, transferts internationaux, cookies, contact DPO. | RGPD art. 13-14 |
| **Conditions générales d'utilisation** | 15 sections | Objet, inscription, contenu utilisateur, propriété intellectuelle, responsabilités, signalement, modération, sanctions, médiation. | DSA (Digital Services Act) |

### 8.1 Accès dans l'application

Les documents sont accessibles depuis l'écran Profil de l'application iOS via trois liens dédiés. Le composant `LegalView` charge les pages via une WebView pointant vers les endpoints de l'API :

- `/legal/mentions?lang=fr|en`
- `/legal/privacy?lang=fr|en`
- `/legal/terms?lang=fr|en`

La langue est détectée automatiquement en fonction des réglages de l'appareil.

**Référence** : `Features/Legal/LegalView.swift`

### 8.2 Points de conformité DSA (Digital Services Act)

Les CGU incluent les dispositions requises par le DSA :

- Mécanisme de signalement de contenu illicite
- Procédure de modération transparente
- Sanctions progressives (avertissement, suspension, suppression)
- Information des utilisateurs sur les décisions de modération
- Voies de recours (médiateur SMP, réf #223702)

---

## 9. Récapitulatif des références de sécurité

| Composant | Fichier(s) source | Description |
|---|---|---|
| JWT RS256 | `utils/jwt.rs` | Génération, validation, claims, key management |
| Hashage Argon2id | `utils/hash.rs` | Paramètres OWASP, hash et verify |
| Politique mdp | `utils/password_policy.rs` | Longueur, mots courants, HIBP |
| HIBP k-Anonymity | `utils/hibp.rs` | Vérification fuites via API |
| Mots de passe courants | `utils/common_passwords.rs` | Base locale 10k mots de passe |
| Auth middleware | `middleware/auth_extractor.rs` | Extraction JWT, blacklist, token_version |
| Auth service | `services/auth_service.rs` | Login, register, refresh, logout, DUMMY_HASH |
| Items handler | `handlers/items.rs` | Filtrage user_id sur chaque opération |
| OG metadata (SSRF) | `services/og_service.rs` | Blocage IPs privées, timeout, taille max |
| Legal views | `Features/Legal/LegalView.swift` | Accès documents légaux in-app |
| Prod checklist | `claudedocs/prod-checklist.md` | Audit de sécurité complet |

---

*Dernière mise à jour : mars 2026*

# DEV-5 — Rapport complet : Migrations PostgreSQL (SQLx)

## 1. Vue d'ensemble

On est parti d'un backend Rust/Axum avec un pool PostgreSQL fonctionnel mais **aucune table**. On a créé 5 migrations réversibles SQLx qui installent le schéma complet d'Offrii : 6 tables, 14 indexes, 2 triggers, 5 CHECK constraints.

### Fichiers touchés

| Fichier | Action | Pourquoi |
|---------|--------|----------|
| `backend/Cargo.toml` | Modifié | Ajout du feature `"migrate"` à sqlx |
| `backend/rest-api/src/main.rs` | Modifié | Auto-run des migrations au démarrage |
| `migrations/20260304000001_create_users.{up,down}.sql` | Créé | Table users + trigger + fonction |
| `migrations/20260304000002_create_categories.{up,down}.sql` | Créé | Table categories + partial indexes |
| `migrations/20260304000003_create_items.{up,down}.sql` | Créé | Table items + indexes + trigger |
| `migrations/20260304000004_create_push_tokens.{up,down}.sql` | Créé | Table push_tokens |
| `migrations/20260304000005_create_circles.{up,down}.sql` | Créé | Tables circles + circle_members |

---

## 2. Comment ça fonctionne

### 2.1 SQLx Migrations — Le mécanisme

SQLx utilise un système de migrations **basé sur des fichiers SQL** dans un dossier `migrations/`. Contrairement à un ORM comme Diesel qui génère du Rust, ici on écrit du SQL pur.

**Le cycle de vie :**

```
Fichier SQL → sqlx migrate run → PostgreSQL exécute le SQL → état tracké dans _sqlx_migrations
```

La table `_sqlx_migrations` (créée automatiquement par SQLx) enregistre :
- Le checksum SHA-256 de chaque fichier SQL
- La date d'application
- Le temps d'exécution

Quand tu relances `sqlx migrate run`, SQLx compare les checksums : si une migration est déjà appliquée, il la saute. C'est **idempotent** — tu peux le lancer 100 fois sans effet.

### 2.2 Naming convention : reversible

SQLx supporte deux formats :
- **Simple** : `{timestamp}_{name}.sql` — migration one-way, pas de revert possible
- **Réversible** : `{timestamp}_{name}.up.sql` + `{timestamp}_{name}.down.sql`

On a choisi le format **réversible** parce que le ticket l'exige (`sqlx migrate revert` doit fonctionner). Le `.up.sql` crée les objets, le `.down.sql` les détruit dans l'ordre inverse.

**Pourquoi des timestamps et pas des numéros séquentiels ?**
SQLx impose le format timestamp pour les migrations réversibles. Les timestamps `20260304000001` à `20260304000005` sont séquentiels et lisibles — la date du jour + un compteur.

### 2.3 Auto-run dans main.rs

```rust
let db = create_pg_pool(&config.database_url).await?;

sqlx::migrate!().run(&db).await?;  // <-- ici
tracing::info!("database migrations applied");
```

La macro `sqlx::migrate!()` :
1. **Au compile-time** : lit les fichiers SQL dans `migrations/` et les embarque dans le binaire (pas besoin du dossier en production)
2. **Au runtime** : compare avec `_sqlx_migrations` et applique les nouvelles

C'est le pattern standard — l'app se met à jour elle-même au démarrage. Si la migration échoue, l'app ne démarre pas (le `?` propage l'erreur et `main()` retourne `Err`).

### 2.4 Le feature `"migrate"` dans Cargo.toml

Sans ce feature flag, ni la macro `sqlx::migrate!()` ni l'outil CLI `sqlx-cli` ne fonctionnent. C'est un choix délibéré de SQLx : le code de migration est optionnel pour ne pas alourdir les builds qui n'en ont pas besoin.

---

## 3. Décryptage de chaque migration

### Migration 001 — `users`

```sql
CREATE OR REPLACE FUNCTION set_updated_at() ...
CREATE TABLE users (...)
CREATE TRIGGER trg_users_updated_at ...
```

**La fonction `set_updated_at()`** est un trigger PostgreSQL écrit en PL/pgSQL. Avant chaque `UPDATE` sur une ligne, PostgreSQL appelle cette fonction qui remplace `updated_at` par `NOW()`. Sans ça, `updated_at` garderait sa valeur initiale pour toujours — ton application devrait manuellement mettre à jour le champ à chaque requête, ce qui est error-prone.

**Pourquoi `CREATE OR REPLACE` ?** Pour être idempotent. Si la fonction existe déjà (improbable dans une migration, mais défensif), elle est simplement remplacée au lieu de crasher.

**Les CHECK constraints sur `reminder_freq`** :
```sql
CHECK (reminder_freq IN ('daily', 'weekly', 'monthly'))
```
Plutôt qu'un ENUM PostgreSQL. Pourquoi ? Les ENUMs PostgreSQL sont **très rigides** : ajouter une valeur nécessite un `ALTER TYPE`, supprimer une valeur est quasiment impossible sans recréer le type. Un CHECK string est modifiable avec un simple `ALTER TABLE ... DROP CONSTRAINT ... ADD CONSTRAINT`.

**`TIMESTAMPTZ` vs `TIMESTAMP`** : On utilise `TIMESTAMPTZ` (avec timezone) partout. C'est la best practice PostgreSQL — les dates sont stockées en UTC et converties selon la timezone de la session. Avec `TIMESTAMP` sans timezone, tu risques des bugs si le serveur change de timezone ou si des utilisateurs sont dans des fuseaux différents.

**`gen_random_uuid()`** : Fonction native PostgreSQL (depuis v13) qui génère des UUID v4. Pas besoin de l'extension `uuid-ossp`.

### Migration 002 — `categories`

```sql
CREATE UNIQUE INDEX uq_categories_user_name
    ON categories (user_id, name) WHERE user_id IS NOT NULL;

CREATE UNIQUE INDEX uq_categories_default_name
    ON categories (name) WHERE user_id IS NULL;
```

**Le problème subtil avec NULL** : En SQL, `NULL != NULL`. Si tu fais un simple `UNIQUE(user_id, name)`, PostgreSQL permet de créer deux catégories `(NULL, "Tech")` parce que `NULL` n'est jamais "equal" à `NULL`. Les catégories par défaut (sans user) pourraient donc avoir des doublons.

**La solution : partial indexes**. Deux indexes séparés :
1. Pour les catégories utilisateur (`user_id IS NOT NULL`) : unicité sur `(user_id, name)`
2. Pour les catégories par défaut (`user_id IS NULL`) : unicité sur `name` seul

C'est un pattern PostgreSQL classique mais souvent méconnu.

**`ON DELETE CASCADE` sur `user_id`** : Si un utilisateur est supprimé, ses catégories custom disparaissent. Les catégories par défaut (`user_id = NULL`) survivent — elles n'appartiennent à personne.

### Migration 003 — `items`

C'est la table centrale d'Offrii (la wishlist).

**Les indexes de performance** :
```sql
CREATE INDEX idx_items_user_status ON items(user_id, status);
CREATE INDEX idx_items_user_priority ON items(user_id, priority);
CREATE INDEX idx_items_created_at ON items(created_at);
```

- `(user_id, status)` : requête la plus fréquente — "afficher mes items actifs". L'index composite permet un index scan au lieu d'un seq scan.
- `(user_id, priority)` : tri par priorité dans la wishlist d'un user.
- `(created_at)` : tri chronologique, pagination.

**`ON DELETE SET NULL` pour `category_id`** : Si une catégorie est supprimée, les items ne sont pas supprimés — leur `category_id` passe à `NULL` (non catégorisé). Différent de CASCADE parce que perdre des items serait catastrophique.

**`DECIMAL(10,2)` pour le prix** : 10 chiffres total, 2 décimales. Permet des prix jusqu'à 99 999 999,99. Jamais utiliser `FLOAT` pour de l'argent — les erreurs d'arrondi sont réelles.

**Soft delete via `status`** : Le status `'deleted'` est un soft delete. L'item n'est pas physiquement supprimé, il est juste marqué. Ça permet :
- L'annulation ("oups, j'ai supprimé par erreur")
- L'historique des achats (`'purchased'`)
- Des stats sur le comportement utilisateur

### Migration 004 — `push_tokens`

Table simple mais avec un détail important :
```sql
UNIQUE(user_id, token)
```
Un utilisateur ne peut pas avoir le même token deux fois (évite les notifications en double), mais peut avoir **plusieurs tokens** (un par device).

### Migration 005 — `circles`

**Deux tables liées** : `circles` (les groupes) et `circle_members` (la table de jointure many-to-many).

```sql
PRIMARY KEY (circle_id, user_id)
```
Composite primary key — un utilisateur ne peut être membre d'un cercle qu'une seule fois. Pas besoin d'un `id` auto-incrémenté ici, la paire est naturellement unique.

**Ordre de suppression dans le `.down.sql`** :
```sql
DROP TABLE IF EXISTS circle_members;  -- d'abord les FK
DROP TABLE IF EXISTS circles;         -- ensuite la table référencée
```
L'ordre inverse de la création. Si on supprime `circles` d'abord, PostgreSQL refuse à cause des foreign keys dans `circle_members`.

---

## 4. Ce qui a été vérifié

| Test | Résultat | Ce que ça prouve |
|------|----------|------------------|
| `sqlx migrate run` | 5/5 appliquées | Les SQL sont valides |
| `\dt` — 7 tables (6 + _sqlx_migrations) | OK | Le schéma est complet |
| `\di` — 14 indexes | OK | Indexes, PK, UNIQUE constraints créés |
| Trigger `updated_at` (INSERT + UPDATE) | `updated_at > created_at = true` | Le trigger fonctionne |
| `sqlx migrate revert` x5 | Toutes les tables supprimées | Les `.down.sql` sont corrects |
| Re-run après revert | 5/5 ré-appliquées | Cycle complet up/down/up fonctionne |
| `cargo build -p rest-api` | Compile | La macro `sqlx::migrate!()` trouve les fichiers |
| `cargo test -p rest-api` | 4/4 tests passent | Pas de régression |

---

## 5. Mon analyse critique

### Ce qui est bien fait

**Réversibilité complète** — Chaque migration peut être annulée proprement. L'ordre de suppression respecte les dépendances FK. On a vérifié le cycle complet up → down → up.

**CHECK constraints plutôt qu'ENUMs** — Choix pragmatique. Quand tu voudras ajouter un status `'archived'` ou une plateforme `'web'`, c'est un simple `ALTER TABLE` au lieu d'une galère avec `ALTER TYPE`.

**Partial indexes sur categories** — Solution correcte au problème NULL/UNIQUE. Beaucoup de développeurs font l'erreur d'un simple `UNIQUE(user_id, name)` et découvrent le bug en production.

**Trigger `set_updated_at()` partagé** — Une seule fonction pour `users` et `items`. DRY au niveau SQL.

**Auto-migration au démarrage** — L'app est auto-suffisante. Un `docker compose up` suffit, pas de script de migration à lancer manuellement.

### Ce qu'on pourrait améliorer

#### 1. Seed data pour les catégories par défaut

Les catégories par défaut (`user_id = NULL`) ne sont jamais créées. Il faudrait une migration 006 (ou un bloc dans 002) :
```sql
INSERT INTO categories (name, icon, is_default, position) VALUES
    ('Tech', 'laptop', TRUE, 0),
    ('Mode', 'shirt', TRUE, 1),
    ('Maison', 'home', TRUE, 2),
    ('Loisirs', 'gamepad', TRUE, 3);
```
Sans ça, un nouvel utilisateur voit une app vide sans catégories.

#### 2. Index manquant sur `items.category_id`

Quand on affiche les items d'une catégorie ou qu'on supprime une catégorie (trigger `ON DELETE SET NULL`), PostgreSQL doit scanner toute la table `items`. Un index accélérerait ça :
```sql
CREATE INDEX idx_items_category ON items(category_id);
```
Pas critique avec peu de données, mais recommandé si la table grossit.

#### 3. Pas de table `sessions` / `refresh_tokens`

Le doc OFFRII-REFERENCE mentionne JWT RS256 avec refresh tokens. Actuellement, aucune table ne stocke les refresh tokens. Soit on fait du stateless pur (pas de révocation possible), soit il faut une table pour tracker les sessions. C'est probablement prévu dans un ticket futur.

#### 4. `purchased_at` non automatique

Quand le status passe à `'purchased'`, `purchased_at` devrait être set automatiquement via un trigger :
```sql
CREATE OR REPLACE FUNCTION set_purchased_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'purchased' AND OLD.status != 'purchased' THEN
        NEW.purchased_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```
Sinon c'est à l'application de le gérer — faisable mais une source potentielle d'oubli.

#### 5. Pas de contrainte d'intégrité circle owner = member

Quand on crée un circle, le `owner_id` devrait automatiquement être inséré dans `circle_members` avec `role = 'owner'`. Actuellement rien n'empêche un owner de ne pas être membre de son propre circle. C'est plutôt de la logique applicative, mais un trigger pourrait garantir la cohérence.

### Ce qui est un choix délibéré (ni bon ni mauvais)

**Soft delete sur items** : Bon pour l'UX (annuler une suppression), mais la table grossit indéfiniment. À terme, il faudra un job de purge ou un archivage. Pour une app personnelle comme Offrii, ce n'est pas un problème avant des années.

**UUID v4 partout** : Bon pour la sécurité (pas de IDs séquentiels devinables) et le distributed computing. Le coût : 16 bytes vs 4 bytes pour un INT, et des indexes légèrement plus gros. Totalement acceptable pour Offrii.

**VARCHAR avec limites** : `email VARCHAR(255)`, `name VARCHAR(100)`, etc. Certains préfèrent `TEXT` sans limite en PostgreSQL (même performance). Les VARCHAR contraints documentent les attentes et protègent contre les abus, c'est un bon choix pour une API publique.

---

## 6. Schéma relationnel final

```
users
  ├── id (PK, UUID)
  ├── email (UNIQUE)
  ├── password_hash
  ├── display_name
  ├── reminder_freq (CHECK: daily/weekly/monthly)
  ├── reminder_time
  ├── created_at
  └── updated_at (TRIGGER)
       │
       ├──< categories (user_id FK, ON DELETE CASCADE)
       │     ├── id (PK, UUID)
       │     ├── name (PARTIAL UNIQUE avec user_id)
       │     ├── icon
       │     ├── is_default
       │     ├── position
       │     └── created_at
       │          │
       ├──< items (user_id FK CASCADE, category_id FK SET NULL)
       │     ├── id (PK, UUID)
       │     ├── name
       │     ├── description, url, estimated_price
       │     ├── priority (CHECK: 1-3)
       │     ├── status (CHECK: active/purchased/deleted)
       │     ├── purchased_at
       │     ├── created_at
       │     └── updated_at (TRIGGER)
       │
       ├──< push_tokens (user_id FK CASCADE)
       │     ├── id (PK, UUID)
       │     ├── token (UNIQUE avec user_id)
       │     ├── platform (CHECK: ios/android)
       │     └── created_at
       │
       ├──< circles (owner_id FK CASCADE)
       │     ├── id (PK, UUID)
       │     ├── name
       │     └── created_at
       │
       └──< circle_members (user_id FK CASCADE, circle_id FK CASCADE)
             ├── (circle_id, user_id) = PK composite
             ├── role (CHECK: owner/member)
             └── joined_at
```

---

## 7. Commandes utiles pour la suite

```bash
# Voir l'état des migrations
DATABASE_URL=... sqlx migrate info

# Appliquer les nouvelles migrations
DATABASE_URL=... sqlx migrate run

# Annuler la dernière migration
DATABASE_URL=... sqlx migrate revert

# Créer une nouvelle migration réversible
DATABASE_URL=... sqlx migrate add -r nom_de_la_migration

# Inspecter les tables PostgreSQL
docker exec offrii-postgres psql -U offrii -d offrii -c "\dt"

# Voir la structure d'une table
docker exec offrii-postgres psql -U offrii -d offrii -c "\d+ users"

# Voir les contraintes
docker exec offrii-postgres psql -U offrii -d offrii -c "
  SELECT conname, contype, pg_get_constraintdef(oid)
  FROM pg_constraint WHERE conrelid = 'items'::regclass;
"
```

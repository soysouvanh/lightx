# API LightX - Guide de démarrage et architecture

Bienvenue dans l'API de démonstration du framework LightX. Ce projet sert de vitrine technologique et pédagogique pour comprendre comment utiliser et interagir avec les incroyables capacités du moteur de génération automatique de code propre à **LightX** et **Daox**.

[English](README.md) | [Français](README.fr.md)

---

## Démarrage rapide (tutoriel pas à pas)

Ce guide est conçu pour être accessible à tous, même aux débutants. Suivez chaque étape minutieusement.

### Étape 1 : Préparer l'environnement de développement

Pour faire tourner ce projet, vous avez besoin des outils de base du langage Rust.

- **Installer Rust** : Si ce n'est pas encore fait, installez le compilateur en ouvrant un terminal et en exécutant la commande officielle :
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  _(⚠️ Indispensable pour les débutants : Une fois l'installation terminée, redémarrez votre terminal ou exécutez `source $HOME/.cargo/env` pour activer la commande `cargo`)._

### Étape 2 : Préparer les bases de données

Ce projet de démonstration montre la puissance de LightX sur 3 bases de données simultanément. L'API nécessite donc que ces bases de données soient joignables pour démarrer.

1. **SQLite** : Fonctionne dans la mémoire vive (`sqlite::memory:`). **Rien à installer !**
2. **PostgreSQL & MySQL** : Le serveur va tenter de s'y connecter avec les identifiants présents dans le fichier `.env` (`localhost:5432` et `localhost:3306`).

   👉 **Solution la plus simple (via Docker)** : Si vous avez Docker d'installé, vous pouvez instantanément créer et démarrer ces deux bases avec de bons identifiants via ces deux commandes :

   ```bash
   # Lancer PostgreSQL
   docker run --name lightx-pg -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lightx_test -p 5432:5432 -d postgres

   # Lancer MySQL
   docker run --name lightx-mysql -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=lightx_test -p 3306:3306 -d mysql
   ```

   _(💡 Si vous n'avez pas Docker, vous pouvez l'installer depuis docker.com. Sinon, vous pouvez exécuter vos propres instances locales et adapter les URLs `mysql://...` et `postgres://...` dans le fichier `.env` du projet)._

### Étape 3 : Démarrer le serveur LightX

Maintenant que Rust est installé et que les bases de données tournent, vous pouvez démarrer l'API.

1. **Placez-vous dans le répertoire du projet `lightx-api` :**

   ```bash
   cd lightx-api
   ```

2. **Lancez le compilateur en développement :**

   ```bash
   cargo run
   ```

   > 💡 **Que se passe-t-il ici ?** Lors de ce démarrage automatisé, le framework va analyser tous vos modèles TOML stockés dans le dossier `schema/`, générer l'entièreté des requêtes SQL (DAO), orchestrer les routeurs (AOP), compiler le tout, puis lancer le puissant serveur asynchrone sécurisé sur le port `3000`.

   > ✅ Vous saurez que tout fonctionne parfaitement lorsque vous verrez apparaitre le message : `Démarrage de LightX API (JSON REST)!`. (Le serveur bloquera ce terminal, c'est normal, il attend vos requêtes).

### Étape 4 : Tester les points de terminaison (endpoints)

Félicitations, le serveur tourne ! Ouvrez maintenant un **tout nouveau terminal** (ou utilisez votre navigateur web) pour appeler les trois routes générées et interagir avec les bases de données.

- **Pour exécuter la démonstration sur PostgreSQL :**
  ```bash
  curl http://localhost:3000/postgres/DbDemo
  ```
- **Pour exécuter la démonstration sur MySQL / MariaDB :**
  ```bash
  curl http://localhost:3000/mysql/DbDemo
  ```
- **Pour exécuter la démonstration sur SQLite :**
  ```bash
  curl http://localhost:3000/sqlite/DbDemo
  ```

🎉 **Résultat attendu :** Chaque requête vous retournera instantanément un tableau de bord JSON détaillé (« status: success »). Ce JSON prouve l'exécution réussie et ultra-rapide de requêtes complexes en tâche de fond (Insertions par lots, Pagination native, Intégrité des Transactions).

_(💡 **Note pédagogique d'import** : Vous vous demandez comment vos conteneurs Docker vides ont obtenu leurs tables ? Lors de cette requête, le Business Object "DbDemoBo" a importé et exécuté automatiquement les fichiers SQL présents dans le dossier `migrations/` avant d'insérer les données d'essai !)_

---

## Tutoriel : créer votre propre API LightX de zéro

Créer une API de bout en bout avec **LightX** est une expérience pensée pour être extrêmement rapide, sécurisée et axée sur l'ingénierie moderne. La philosophie du framework est la suivante : _Déclaratif d'abord, Code Métier pur, et Zéro Reflection à l'exécution_.

Voici le guide pédagogique pas-à-pas.

### Phase 1 : Architecture et fondations

**1. Initialisation du projet Rust**
On commence par créer le projet standard :

```bash
cargo new my_lightx_api
cd my_lightx_api
```

**2. Les dépendances (`Cargo.toml`)**
Ouvrez le fichier `Cargo.toml` et ajoutez les bibliothèques requises. `lightx` apporte le serveur asynchrone, et `daox` permet la génération de code.

```toml
[dependencies]
lightx = "0.1"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres"] }
dotenvy = "0.15"

[build-dependencies]
daox = { version = "0.2", features = ["postgres"] }
lightx = "0.1"
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
```

**3. Les Variables d'environnement (`.env`)**
À la racine de votre projet, créez un fichier `.env`. C'est le seul endroit où renseigner vos accès réseau.

```env
POSTGRES_DATABASE_URL=postgres://user:password@localhost:5432/ma_base
```

### Phase 2 : Modélisation des données (la couche DAO)

La force absolue de LightX réside dans son générateur **Daox**, qui adopte une approche inébranlable _Database-first_.

**1. Créer la table en SQL**
Créez votre table physiquement dans votre base Postgres via votre client SQL habituel :

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL
);
```

**2. Indiquer la cible (`schema/databases.toml`)**
Créez le dossier `schema/` puis le fichier `schema/databases.toml` pour indiquer au compilateur quelle base lire :

```toml
[postgres]
dialect = "postgres"
```

**3. Le script de génération (`build.rs`)**
À la racine du projet (au même niveau que `Cargo.toml`), créez le script `build.rs` qui va introspecter vos tables et créer le Routeur avant la phase de compilation :

```rust
use daox::DaoGenerator;
use lightx::core_generator::CoreGenerator;
use lightx::handler_generator::HandlerGenerator;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let out_dir = env::var("OUT_DIR").unwrap();

    // 1. Introspection et génération du DAO
    let dao_gen = DaoGenerator::new("schema", &out_dir);
    if let Ok(url) = env::var("POSTGRES_DATABASE_URL") {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let _ = dao_gen.introspect(&url).await;
        });
    }
    dao_gen.generate_dao()?;

    // 2. Génération du Routeur (Handlers & Core)
    HandlerGenerator::new("handlers", &out_dir).generate_handlers()?;
    CoreGenerator::new("handlers", "i18n", &out_dir).generate_core()?;

    Ok(())
}
```

### Phase 3 : Création du routeur via l'AOP (handlers)

Contrairement aux frameworks classiques, LightX utilise l'Aspect-Oriented Programming de façon déclarative, éliminant tout code "spaghetti".

**Déclarer la route (`handlers/CreateUser.toml`)**
Créez le dossier `handlers/`, puis le fichier `handlers/CreateUser.toml` :

```toml
name = "CreateUser"
method = "POST"
path = "/users/create"
bo_class = "crate::bo::create_user::CreateUserBo"
```

À la compilation, le framework lira ce fichier pour générer un contrôleur sécurisé qui parsera la requête HTTP et appellera votre logique métier.

### Phase 4 : Écrire la logique métier (BO)

Oubliez la gestion réseau, concentrez-vous sur l'algorithme.

**1. Déclarer le module**
Créez un dossier `src/bo/` et créez le fichier `src/bo/mod.rs` pour exposer le module :

```rust
pub mod create_user;
```

**2. Écrire le traitement (`src/bo/create_user.rs`)**
Créez ensuite le fichier `src/bo/create_user.rs` désigné par votre Handler :

```rust
use lightx::core::AppError;
use lightx::ext::hyper::Response;

pub struct CreateUserBo;

impl CreateUserBo {
    pub async fn execute(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<String>, AppError> {

        // 1. Connexion paresseuse à la base de données via le contexte centralisé
        let pool = &ctx.postgres_pool;

        // 2. Utilisation du modèle Rust natif (Généré automatiquement par Daox)
        let new_user = crate::PostgresUsers {
            id: 0,
            email: "mon_email@domaine.com".to_string(),
        };

        // 3. Insertion avec gestion d'erreur stricte par le framework
        let user_id = new_user.insert(pool).await.map_err(|e| AppError::DatabaseError {
            msg: e.to_string(), file: file!(), line: line!()
        })?;

        // 4. Renvoi formel de la réponse HTTP
        Ok(Response::new(format!("Utilisateur créé avec l'ID {}", user_id)))
    }
}
```

### Phase 5 : L'entrée du programme (`src/main.rs`)

Ouvrez votre fichier `src/main.rs` et remplacez son contenu par le code de démarrage. C'est ici que l'on inclut l'intégration des fichiers autogénérés.

```rust
use std::sync::Arc;
use lightx::server;

// Inclusion stricte du code natif généré lors du build (DAO, Handlers, Core)
include!(concat!(env!("OUT_DIR"), "/daox_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_handlers_generated.rs"));
include!(concat!(env!("OUT_DIR"), "/lightx_core_generated.rs"));

// L'arborescence contenant votre logique métier
pub mod bo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok(); // Charge les variables du réseau
    lightx::logger::init("./log"); // Lancement des logs asynchrones

    lightx::logger::info("Démarrage de l'API Propulsée par LightX !".to_string());

    // Context Factory : Initialise directement les pools de bases de données
    let factory = Arc::new(crate::AppContextFactory::new().await?);

    // Le routeur précompilé statiquement (sans aucun coût d'exécution O(1))
    let router = Arc::new(crate::AppRouter { factory: factory.clone() });

    // Démarrage de l'écoute sur le port 3000
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    server::listen(addr, factory, router).await?;

    Ok(())
}
```

Lancez simplement `cargo run`, et votre API de très haute performance est en ligne !

---

## Comprendre le framework : excellence et architecture

Ce projet ne se contente pas des traditionnelles API REST. Il vise à inculquer les pratiques de l'**Excellence Pédagogique et Technologique** adoptées par LightX.
Voici tout ce qu'il vous faut pour démystifier le rôle des différentes couches et développer en parfaite sécurité.

![Architecture Globale](assets/architecture_fr.svg)

### 1. La puissance des macros (zéro surcout)

LightX prohibe formellement la _réflexion informatique classique_ (l'introspection lente pendant l'exécution qu'utilisent beaucoup de frameworks).
Au moment précis où vous exécutez `cargo run`, le moteur lit vos `TOML` et `SQL` pour forger à la volée du **pur code de bas niveau Rust** hyper-optimisé (via _Daox_).

- **Conséquence en Production** : Routage foudroyant (Vitesse `O(1)`), sécurité mémoire absolue (absence de fuites imprévues) et un code strictement immunisé natif face aux vulnérabilités d'injection SQL.

### 2. La magie de l'orienté aspect (AOP)

Dans ce dépôt, vous remarquez le sous-dossier `handlers/` et ses fichiers TOML. C'est eux qui commandent l'AOP.
Le framework génèrera un super-contrôleur imperméable à toute faille qui se charge :

- D'analyser le JSON entrant et l'URL selon les règles strictes.
- D'assurer l'aiguillage le plus direct vers la bonne fonction.
- D'exécuter l'enchainement de vos **Objects Métiers / Business Objects (BO)**.
- Option vitale : De décider souverainement s'il émet un `COMMIT` ou s'il interrompt tout sur un `ROLLBACK` via SQL (en cas d'erreur minime sur n'importe quel traitement).

![Flux AOP](assets/aop_fr.svg)

### 3. Écosystème des couches (séparation rigueur)

Chaque brique a une responsabilité immuable, conçue pour vous faciliter la vie (Accessibilité) ;

1. **La Couche d'Accès aux Données (DAO)** : Générée par nos librairies (Daox). Elle exécute le gros-œuvre invisible et vous offre des fonctions prêtes à l'emploi (Insert, UpSert, Cursors stream...).
2. **La Couche Métier (BO)** : Située dans le dossier `src/bo/`. **C'est précisément ici que vous écrivez votre code !** Affranchissez-vous du réseau et des requêtes pures en appelant directement le DAO pour implémenter tranquillement vos algorithmes métier.
3. **Le RequestContext (Le Bus)** : LightX n'utilise jamais d'état global risqué. La Context Factory rassemble vos `.env` (ex: `ctx.postgres_pool`) en un environnement paresseux ultra léger, livré directement à vos méthodes métiers (BO) et supprimé magistralement de la mémoire via les règles temporelles Rust (RAII) dès la fin de requête HTTP.

![Cycle de vie du Contexte](assets/context_fr.svg)

### 4. Zero panic, 100% Rust sécuritaire

Sur l'architecture LightX, vous allez interagir au quotidien avec l'erreur `AppError` visible (par exemple dans le fichier métier expérimental `DbDemoBo.rs`).
L'intention globale bannit rigoureusement l'usage inattendu des crasheurs tels que `panic!()` ou `unwrap()`.

**Le mécanisme natif de résilience** : Si une boucle métier panique, qu'un SQL renvoie une ligne manquante, l'erreur pure est captée de plein vol. L'énumérateur `AppError` va traduire en langage universel l'exception via `?`, recracher une formidable réponse JSON structurée au Frontend, et votre serveur REST tiendra sa promesse : continuer à servir la population des autres clients sans jamais arrêter l'exécutable natif !

---

**Félicitations**, vous commencez l'aventure avec les certitudes et les secrets internes de la machinerie universelle et foudroyante de LightX !

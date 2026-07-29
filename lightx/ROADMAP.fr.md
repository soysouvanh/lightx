# LightX Roadmap : Performances pures, sécurité militaire & accessibilité novice

Bien que l'architecture actuelle de **LightX** pose de solides fondations innovantes (_Database-First_, _Shift-Left Generation_, exécution _O(1)_), notre feuille de route se concentre exclusivement sur six piliers vitaux :

1. **L'accessibilité novice** : Offrir une fluidité totale d'apprentissage pour les développeurs débutants dans l'écosystème Rust.
2. **La qualité artisanale** : Maintenir une base de code d'une modularité parfaite, 100% testable, et ultra-maintenable.
3. **La précision chirurgicale militaire** : Garantir une sécurité applicative infaillible, au pixel près.
4. **La performance absolue** : Pousser la vitesse d'exécution à son paroxysme SOTA (State of the Art).
5. **L'extensibilité cloud-native** : Assurer une résilience et interopérabilité totale avec les infrastructures critiques (Microservices, très haute concurrence asynchrone _Tokio_, Observabilité OpenTelemetry).
6. **L'écosystème outillé ("batteries-included")** : Livrer nativement l'arsenal DevOps permettant une mise en production instantanée (CLI intégrée, migrations de schémas de données, OpenAPI généré sans surcoût).

Voici la cartographie des chantiers stratégiques restants pour achever cette vision d'excellence technique et pédagogique.

---

## Phase 0 : Refactoring SOTA (goulet d'étranglement & zéro-allocation)

<details>
<summary>[x] 0.1. Éradication de l'allocation mémoire JSON (deserialization zero-copy)</summary>

- **Objectif :** Supprimer la cascade d'allocations mortelles sur la Heap liée à l'utilisation hybride de `HashMap<String, String>` (converti depuis `serde_json::Value`) lors du parsing HTTP.
- **Contexte code source :** La struct `RequestContext` est générée par **`dao_generator.rs`** (ligne 677 : `pub struct RequestContext`), pas par `core_generator.rs`. Elle contient `pub raw_parameters: std::collections::HashMap<String, String>` (ligne 679) et `pub _bus: lightx::core::TypeMap` (ligne 678). Parallèlement, **`core_generator.rs`** duplique intégralement ce parsing dans les fonctions `start_server` (lignes 260-280) et `start_server_tls` (lignes 433-453) qu'il génère : ces blocs re-parsent chacun `serde_json::from_slice::<HashMap<String, serde_json::Value>>` puis convertissent chaque `Value` en `String`. Dans `server.rs`, le même parsing existe lignes 67-74 via `serde_json::from_slice` (mais typé `HashMap<String, serde_json::Value>` directement). `handler_generator.rs` accède au résultat via `ctx.raw_parameters.get("...")` (ligne 364).
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`dao_generator.rs` ligne 679) :** Dans la génération de `RequestContext`, remplacer `pub raw_parameters: std::collections::HashMap<String, String>` par `pub raw_body: bytes::Bytes`. Adapter les constructeurs `new()` (ligne 700) et `new_test_context()` (ligne 715) pour initialiser `raw_body: bytes::Bytes::new()` au lieu de `raw_parameters: std::collections::HashMap::new()`.
  - **Étape 2 (`server.rs` lignes 61-74) :** Supprimer le bloc de parsing JSON intermédiaire (`serde_json::from_slice` + `HashMap::new()`). Injecter directement les octets collectés dans le contexte. Retirer `use std::collections::HashMap;` (ligne 1) devenu mort.
  - **Étape 3 (`handler_generator.rs` ligne 364) :** Cesser de générer `ctx.raw_parameters.get("...")`. Le code AOP généré devra désormais désérialiser depuis `ctx.raw_body` vers la struct `Payload` typée. S'assurer que `#[derive(serde::Deserialize)]` soit systématiquement annoté sur ces structs.
  - **Étape 4 (`core_generator.rs` lignes 260-280 et 433-453) :** Puisque ces blocs seront supprimés en 0.4.1, cette étape est couverte par la purge de `start_server`/`start_server_tls`. Valider qu'aucune trace résiduelle de `raw_parameters` ne subsiste dans le routeur `route_request` (ligne 168).
- **Critère d'Acceptation :** L'usage instable de `ctx.raw_parameters` disparaît intégralement (0 occurrence dans le code source ET le code généré). Baisse colossale chiffrée de l'empreinte RAM sous un benchmark asynchrone (Outil `bombardier` / `oha`).

</details>

<details>
<summary>[x] 0.2. Bouclier anti-OOM (out of Memory) & connexion infinie</summary>

- **Objectif :** Interdire militairement le crash matériel si un attaquant tente un Buffer Overflow réseau avec une charge virtuellement infinie ou une attaque Slowloris.
- **Contexte code source :** `server.rs` ligne 61 : `req.into_body().collect().await` sans aucune limite de taille. `server.rs` lignes 171, 223 : `http1::Builder::new().serve_connection(io, service).await` sans timeout. `core_generator.rs` lignes 250, 328, 423, 501 : mêmes vulnérabilités dupliquées dans le code généré `start_server`/`start_server_tls`.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`server.rs` ligne 61) :** Remplacer le chargement de corps non-sécurisé `req.into_body().collect().await` par le limiteur strict importé : `http_body_util::Limited::new(req.into_body(), 81920).collect().await`. Si rejet `LengthLimitError`, renvoyer un statut `HTTP 413`.
  - **Étape 2 (`server.rs` lignes 171, 223) :** Sur les appels `http1::Builder::new().serve_connection(io, service).await` dans `listen` et `listen_tls`, encapsuler avec `tokio::time::timeout(std::time::Duration::from_secs(5), ...)`. Si `Elapsed`, supprimer la connexion silencieusement.
- **Critère d'Acceptation :** Une exécution soumise à 80Ko+ déclenche un brutal `HTTP 413 Payload Too Large`. Une connexion fantôme est tuée net passé un délai défini (ex: 5 secondes), libérant immédiatement la RAM et le thread asynchrone.

</details>

<details>
<summary>[x] 0.3. Purge du bus polymorphique dynamique (Box<dyn Any>)</summary>

- **Objectif :** Rayer définitivement les calculs CPU de virtualisation des pointeurs (Downcast) induits par le `TypeMap`.
- **Contexte code source :** `TypeMap` est défini dans `core.rs` lignes 15-101 (struct + impls). Il est injecté dans `RequestContext` par `dao_generator.rs` ligne 678 (`pub _bus: lightx::core::TypeMap`) et instancié lignes 699, 714 (`_bus: lightx::core::TypeMap::new()`). Il est aussi dupliqué dans `core_generator.rs` lignes 280, 453. `server.rs` l'importe ligne 21 et l'instancie ligne 79. `handler_generator.rs` l'utilise ligne 620 (`ctx._bus.insert::<...>(payload)`).
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`dao_generator.rs` lignes 678, 699, 714) :** Dans la génération de `RequestContext`, supprimer le champ `pub _bus: lightx::core::TypeMap` et ses instanciations `_bus: lightx::core::TypeMap::new()`.
  - **Étape 2 (`server.rs` ligne 79) :** Retirer `_bus: TypeMap::new()` de l'instanciation du contexte. Retirer `TypeMap` de l'import ligne 21.
  - **Étape 3 (`handler_generator.rs` ligne 620) :** Supprimer la ligne `ctx._bus.insert::<...>(payload)`. Refactoriser le passage de la payload en paramètre explicite et typé des méthodes BO.
  - **Étape 4 (`core.rs`) :** Si après les étapes 1 à 3, un `grep -r "TypeMap"` sur le crate confirme zéro usage résiduel, supprimer intégralement la struct `TypeMap` et ses impls (lignes 15-102).
- **Critère d'Acceptation :** Disparition totale de `ctx._bus` à l'exécution ; les objets d'états sont alloués directement au cœur des signatures de fonctions.

</details>

<details>
<summary>[x] 0.4. Démantèlement du "dead-code" shift-left & dépendances lourdes</summary>

- **Objectif :** Diviser mécaniquement la consommation (AST) du build par le compilateur, purger les paquets morts, et annihiler la friction multithread.
- **Contexte code source :** `core_generator.rs` génère `pub async fn start_server` (ligne 208) et `start_server_tls` (ligne 336) en String littérale dans `OUT_DIR`. `handler_generator.rs` injecte `lazy_static!` (lignes 437, 543) et `#[inline(always)]` (lignes 358, 631). `dao_generator.rs` injecte également `#[inline(always)]` (lignes 819, 854, 884, 927). `Cargo.toml` déclare `tokio = { features = ["full"] }` (ligne 24) et `lazy_static = "1.4"` (ligne 38). `lib.rs` réexporte `lazy_static` (ligne 26) et `regex` (ligne 27).
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`core_generator.rs` et `server.rs`) :** Éliminer la génération archaïque en String littérale des méthodes `start_server` (lignes 206-335) et `start_server_tls` (lignes 336-506) dans `core_generator.rs`. Ces blocs parasitent le code généré. Afin d'utiliser le module natif `server.rs` (qui est hélas un fichier mort obsolète non exporté dans `lib.rs` et contenant une erreur de compilation historique sur `RequestContext`), il est exigé de **ressusciter `server.rs`** : abstraire les dépendances à `RequestContext` via un type générique `C: ContextFactory` pour supporter le multi-bases dynamique, exporter `pub mod server;` dans `lib.rs`, puis effacer les ~300 lignes du générateur.
  - **Étape 2 (`Cargo.toml` lignes 4, 24) :** Conserver `edition = "2024"` (stable depuis février 2025), mais épingler formellement le MSRV en ajoutant `rust-version = "1.85"`. Remplacer scrupuleusement `tokio = { version = "1.0", features = ["full"] }` (ligne 24) par `tokio = { version = "1.0", features = ["rt-multi-thread", "macros", "net", "time"] }`.
  - **Étape 3 (`handler_generator.rs` lignes 437, 439, 543, 546) :** Remplacer chaque bloc `lazy_static!` généré par `std::sync::LazyLock::new(|| ...)`. Attention : le code généré actuel utilise un `.unwrap()` dans la Regex compilée à l'intérieur du `lazy_static!` (ligne 439 et 546 : `Regex::new(...).unwrap()`). Transposer ce schéma vers `LazyLock` pour supprimer cet `unwrap()` au profit d'une Regex validée statiquement à la compilation par le générateur (qui vérifie déjà la syntaxe de la regex lignes 316-323).
  - **Étape 4 (`handler_generator.rs` lignes 358, 631 et `dao_generator.rs` lignes 819, 854, 884, 927) :** Supprimer formellement chaque attribut `#[inline(always)]` au-dessus de toute méthode `async` générée (6 occurrences totales confirmées dans le code source).
  - **Étape 5 (`Cargo.toml` ligne 38 et `lib.rs` ligne 26) :** Retirer `lazy_static = "1.4"` des dépendances et supprimer `pub use lazy_static;` de la réexportation `ext`.
  - **Étape 6 (`Cargo.toml` ligne 37 et `lib.rs` ligne 27) :** Auditer si la dépendance `regex = "1.10"` est encore nécessaire après la migration `LazyLock`. Si les regex AOP sont pré-validées à la compilation et intégrées via `LazyLock`, le crate `regex` peut rester, mais `pub use regex;` dans `ext` (ligne 27) doit être retiré **seulement si** le code généré est refactorisé pour ne plus passer par `lightx::ext::regex`. Dans le cas contraire, conserver la réexportation.
- **Critère d'Acceptation :** Un `cargo build` répété sur codebase inchangée s'exécute en une fraction de seconde, le code généré est purgé, le binaire supprime ses contentions Regex (Locks) et son poids chute significativement.

</details>

<details>
<summary>[x] 0.5. Rust SecOps Immediate Patching (vulnérabilités critiques)</summary>

- **Objectif :** Colmater les brèches de sécurité critiques du noyau actuel et installer des fondations imperméables.
- **Contexte code source :** `server.rs` lignes 112-113 transmettent le `msg` brut de `DatabaseError`/`SystemError` directement au client. Lignes 103, 106, 110, 113 construisent du JSON par interpolation `format!`. Lignes 125-154 (`bad_request`, `internal_error`, `not_found`) utilisent aussi des `format!` manuels. Lignes 186, 192 contiennent deux `.expect()` fatals. Ligne 204 annonce ALPN `h2` sans servir HTTP/2. `dotenv` n'est pas dans le `Cargo.toml` actuel (audit confirmé) mais peut exister dans le code utilisateur généré.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`server.rs` lignes 112-113) :** Sur le match `AppError::DatabaseError | AppError::SystemError`, remplacer `internal_error(&format!("{{\"error\":\"{}\"}}", msg))` par : appel à `logger::error(msg)` puis retour d'un payload fermé `serde_json::json!({"code": 500, "error": "Internal Server Error"}).to_string()`. La payload originale SQLx ne doit jamais toucher la réponse HTTP.
  - **Étape 2 (`server.rs` lignes 90, 103, 106, 110, 125-154) :** Détruire systématiquement toutes les constructions par interpolation textuelle (`format!("{{\"field\":\"{}\",\"msg\":\"{}\"}}", ...)` ou `format!("{{\"error\":\"{}\"}}", ...)`) au profit exclusif et rigoureux de la macro `serde_json::json!({...}).to_string()`.
  - **Étape 3 (`server.rs` lignes 186, 192) :** Éradiquer les deux `.expect()` en les remplaçant par un `match` ou un opérateur `?` renvoyant l'erreur proprement. Un framework Zéro-Panic ne peut pas contenir d'`.expect()` dans sa couche réseau.
  - **Étape 4 (`Cargo.toml`) :** Confirmer l'absence de `dotenv` (audit validé). Si des traces existent dans le code généré, les remplacer par `dotenvy`.
  - **Étape 5 (`server.rs` ligne 204) :** Mettre l'ALPN au diapason. Soit implémenter `hyper::server::conn::http2::Builder` pour honorer `h2`, soit retirer `b"h2".to_vec()` de `config.alpn_protocols` pour ne conserver que `http/1.1`.
  - **Étape 6 (`server.rs` lignes 96, 125-154) :** Injecter statiquement les headers de sécurité dans la réponse de succès (ligne 96) et les fonctions utilitaires `bad_request`, `internal_error`, `not_found` : `.header("Strict-Transport-Security", "max-age=63072000").header("X-Content-Type-Options", "nosniff").header("X-Frame-Options", "DENY")`.
  - **Étape 7 (`server.rs` - `listen_tls` ligne 201) :** Forcer `rustls` pour n'accepter exclusivement que TLS 1.3, via `ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])`.
- **Critère d'Acceptation :** Audit zéro-faille au scan local et scoring A+ SSL Labs aux tests des headers de sécurité. Aucune information de la topologie base de donnée n'est extraite lors d'un crash API. Le binaire ne contient plus aucun `.expect()` ou `.unwrap()` dans la couche réseau.

</details>

<details>
<summary>[x] 0.6. Convergence Standard tower::Service</summary>

- **Objectif :** Ne pas recoder l'observabilité réseau, mais rendre le noyau immédiatement compatible avec l'écosystème `tower-http` (CORS, Tracing, Timeout natif).
- **Contexte code source :** `LightXService` implémente déjà `hyper::service::Service<Request<Incoming>>` (import `server.rs` ligne 11, impl ligne 47). Cependant, le `type Future` associé est déclaré `Pin<Box<dyn Future<...> + Send>>` (ligne 50), ce qui implique une allocation Heap par requête. Ce `Service` est de Hyper, pas de Tower. Tower possède son propre trait `tower::Service` avec une signature compatible mais distincte.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`Cargo.toml`) :** Ajouter la dépendance standard `tower = { version = "0.5", features = ["util"] }` et `tower-http = { version = "0.6", features = ["cors", "trace"] }`. **Note :** Tower 0.5 est la version stable current en 2025 ; vérifier la dernière version avant implémentation.
  - **Étape 2 (`server.rs` ligne 47) :** Remplacer l'impl `hyper::service::Service` par `tower::Service<Request<Incoming>>`. Tower et Hyper partagent le même trait contract depuis la convergence Hyper 1.x / Tower 0.5. Migrer le `type Future` pour éliminer le `Box<dyn Future>` en faveur d'un type concret (opaque via `impl Future` ou type associé nommé), ce qui supprimera l'allocation Heap par requête.
  - **Étape 3 (`server.rs` ou `core_generator.rs`) :** Exposer la capacité pour le développeur de chaîner trivialement ses `ServiceBuilder` via Tower sans écrire de code propriétaire, le tout résolu à la compilation.
- **Critère d'Acceptation :** La possibilité d'activer nativement un CORS via `tower_http::cors::CorsLayer` englobant le `LightXService` sans conflit. Le `Box<dyn Future>` de la ligne 50 disparaît.

</details>

---

## Phase 1 : Expérience développeur maximale (DX & batteries-included)

<details>
<summary>[x] 1.1. Auto-génération stricte de l'OpenAPI (Zéro-Parsing d'Exécution)</summary>

- **Objectif :** Transformer le contrat des Business Objects et Validators statiques en un fichier JSON OpenAPI monolithique (Swagger) généré _pendant_ la phase de compilation O(1), afin de s'affranchir de toute traversée d'arbre (AST) ou parsing en mode _runtime_ propre aux autres frameworks.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`handler_generator.rs`) :** Tirer profit de l'existant. Infiltrer l'extracteur de configuration où un parsing TOML des dictionnaires AOP a _déjà_ eu lieu à l'évaluation `build.rs`.
  - **Étape 2 (`handler_generator.rs`) :** Conceptuellement transposer cette hiérarchie détectée en une structure JSON validant l'OAS (OpenAPI 3.0), à plat. Sauver la sortie brute produite proprement sous `$OUT_DIR/openapi.json`.
  - **Étape 3 (`core_generator.rs`) :** Injecter virtuellement dans le radix rooter statique O(1) la route immatérielle `GET /swagger`, programmant le renvoi monolithique de la chaîne (`include_str!(concat!(env!("OUT_DIR"), "/openapi.json"))`). Zéro chargement serveur.
- **Critère d'Acceptation :** Le framework devra intercepter et exposer par lui-même la racine virtuelle `GET /swagger` servant statiquement ce fichier d'architecture, éliminant de fait tout besoin de désérialisation instable à l'exécution.
</details>

<details>
<summary>[x] 1.2. Console d'automatisation en ligne de commande (lightx-cli)</summary>

- **Objectif :** Fournir le chaînon manquant du scaffolding immédiat, déployable pour le grand public.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`workspace` / `lightx-cli`) :** Initier un nouveau crate Rust de type CLI binaire (par ex: utilisant `clap`) nommé `lightx-cli`.
  - **Étape 2 (`lightx-cli/src/main.rs`) :** Coder une procédure `lightx new <nom_projet>` instanciant les répertoires minimaux structurels exigés (`schema/`, `handlers/`, `src/bo/`) et écrivant un gabarit noyau `Cargo.toml`.
  - **Étape 3 (`lightx-cli`) :** Implémenter le _scaffold automatisé_ `lightx add handler <Method> <URI>` générant le masque TOML de pare-feu requis, et le fragment `*.rs` de Business Object associé. Aucune création logicielle au clavier ne doit subsister.
- **Critère d'Acceptation :** Impossibilité d'échec pour un nouvel ingénieur démarrant une conception de zéro (aucune configuration humaine de cargo initiale complexe requise).
</details>

<details>
<summary>[x] 1.3. Simulateur sandbox de testing end-to-end natif</summary>

- **Objectif :** Exécuter des tests d'intégration destructeurs en boucle sur le code métier sans altérer physiquement le flux des données structurant le projet.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`workspace` & `lightx-macro`) :** Créer un sub-crate `lightx-macro` target `proc-macro = true` dans le workspace pour héberger les macros procédurales.
  - **Étape 2 (`lightx-macro`) :** Forger automatiquement via macro des conteneurs _mockés_ et un système hermétique d'injection de transaction.
  - **Étape 3 (`tests/`) :** Exposer formellement pour les `tests` unitaires l'attribut spécial `#[lightx_macro::test]` injectant avant compilation sa propre simulation logicielle de contexte réseau (`RequestContext`) et de pool asynchrone protégé.
- **Critère d'Acceptation :** Obligation stricte de `ROLLBACK` total inconditionnel sur toute la portée en fin de test. La base du développeur reste vierge.
</details>

<details>
<summary>[x] 1.4. Pipeline CI/CD Strict & Découplage Build (offline Mode)</summary>

- **Objectif :** Forcer virtuellement l'équipe à affronter son code via intégration continue inviolable et exhaustive.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`.github/workflows/ci.yml`) :** Initialiser la chaîne d'action virtuelle embarquant nativement : `cargo clippy -- -D warnings`, un test formateur complet `cargo fmt --check`, le module de surveillance `cargo-audit`...
  - **Étape 2 (`Cargo.toml` et CI) :** Intégrer formellement une **matrice de versions CI** vérifiant explicitement que la compilation passe sur le MSRV (`1.85`) et sur `stable`. Activer formellement les lints paranoïaques au sommet du `lib.rs` racine (`#![deny(clippy::all, clippy::pedantic, missing_docs)]`) et exiger un rapport de couverture >90% via `cargo-tarpaulin`. _(Impératif : Les générateurs devront injecter statiquement un `#[allow(clippy::pedantic, missing_docs)]` sur l'AST cible pour ne pas bloquer les CI clientes lors de l'include! du code généré)_.
  - **Étape 3 (`.github/workflows/ci.yml`) :** ...et greffer de force un _Container Service MySQL_ (ex: image Docker `mysql:8` avec paramétrage mot de passe racine d'environnement).
  - **Étape 4 (`dao_generator.rs`) :** L'usage de `cargo sqlx prepare` étant technologiquement incompatible avec le code généré dans `OUT_DIR`, remplacer formellement les appels macro purs `sqlx::query!` (et `sqlx::query_as!`) par leurs versions dynamiques d'exécution (`sqlx::query().bind(...)` et `sqlx::query_as().bind(...)`). Le typage structurel strict étant DÉJÀ certifié mathématiquement par le générateur d'après le schéma, l'usage dynamique garantit une sécurité équivalente.
  - **Étape 5 (`tests/`) :** Rédiger des Tests Unitaires dédiés exclusifs aux processeurs de syntaxe `dao_generator.rs` et `core_generator.rs`, validant via simple module Rust l'exactitude des fichiers générés.
- **Critère d'Acceptation :** Rejet absolu du code mort/fuites sur branche principale. Le `build.rs` n'échoue plus jamais sur environnement dissocié et les codes générés n'exigent plus formellement de base de données active à la compilation (Offline Build natif absolu).
</details>

<details>
<summary>[x] 1.5. Cœur Artisanal : Migrations DB & Versioning Strict</summary>

- **Objectif :** Industrialiser la gestion de données et projeter LightX sur la scène publique.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`lightx-cli`) :** Éliminer le code brut unitaire `init_mysql.sql` (qui est actuellement un simple fichier de dump) au profit d'une commande `lightx migrate` pilotant un véritable répertoire structuré de migrations SQL versionnées et rejouables, compatible O(1).
  - **Étape 2 (`Cargo.toml`) :** Uniformiser le versioning en _SemVer_ absolu et obliger la lecture statique du `CHANGELOG.md` à la compilation.
- **Critère d'Acceptation :** Parution de LightX sur `crates.io` avec badge actif et doc certifiée sur `docs.rs`.
</details>

<details>
<summary>[x] 1.6. Rust DX Pédagogique absolue (docs & erreurs de compilation)</summary>

- **Objectif :** Livrer une expérience d'embarquement (Onboarding/DX) surpassant les standards par défaut en tirant parti du compilateur Rust.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`/lightx-examples`) :** Créer physiquement un répertoire racine `/lightx-examples` structuré **sous forme de workspace logiciel ou de Crates autonomes isolés** (ex: `/lightx-examples/01_hello_world/Cargo.toml`) pour au moins 3 applicatifs (`01_hello_world`, `02_database_crud`, `03_jwt_auth`). En raison de l'architecture _Shift-Left_, n'utilisez pas de simples fichiers pour la cible native `--example` de Cargo (ils entreraient en conflit avec le générateur). L'exécution se fera via `cargo run -p <nom>` (ou `cd`) en émulant un projet complet avec son propre `build.rs`.
  - **Étape 2 (`src/*.rs`) :** Apposer obligatoirement des _doc-tests_ exécutables partout au sommet des implémentations publiques pour constituer le socle pédagogique primaire de `docs.rs` (ces tests couvriront les structures statiques, en tenant compte que le code AOP ciblé reste généré).
  - **Étape 3 (Macros & Core) :** Adopter l'attribut `#[diagnostic::on_unimplemented("Votre message ultra-précis")]` sur les traits complexes critiques (ex: contrat de retour du Business Object, `IntoLightXResponse`) pour formater conceptuellement les erreurs du compilateur.
  - **Étape 4 (`docs/`) :** Initialiser formellement un site statique pédagogique global via le standard Rust `mdbook`, exposant l'architecture via des diagrammes animés Mermaid complets. Automatiser le déploiement via CI/CD.
- **Critère d'Acceptation :** Le compilateur Rust guidera lui-même un développeur novice si ce dernier implémente mal une architecture, et l'outil possèdera son propre site web d'apprentissage copiable.
</details>

---

## Phase 2 : Robustesse cloud-native et sécurité militaire (SecOps)

<details>
<summary>[x] 2.1. Middleware de frappe chirurgicale anti-DDOS & rate limiting</summary>

- **Objectif :** Prémunir l'intégrité de la machine en rejetant massivement et en `O(1)` tout trafic abusif, bien avant qu'il ne puisse allouer de lourds conteneurs de donnée.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`server.rs`, `dao_generator.rs` & `handler_generator.rs`) :** **[Correction Architecturale]** L'ajout direct au `LightXService` viole l'abstraction générique développée (Phase 0.4.1). Forcer `dao_generator.rs` à instancier le registre concurrent de limiteurs (pool `moka` ou `redis`) directement au cœur du `ContextFactory` généré, qui l'injectera (`pub rate_limiter: Arc<Cache>`) au `RequestContext`. Afin d'isoler l'attaquant, ajouter l'empreinte logicielle (`pub client_ip: std::net::IpAddr`) au contexte. Mettre à jour `server.rs` pour capturer l'IP réseau (`io.peer_addr()`) du flux TCP et la transmettre au `ContextFactory` (qui donnera préséance aux entêtes `X-Forwarded-For` / `X-Real-IP`). Imposer ensuite à `handler_generator.rs` de générer l'algorithme AOP strict Token Bucket (par IP) au sommet de la route.
  - **Étape 2 (`handlers/*.toml`) :** Rendre expressif par un simple switch explicatif via un paramètre du profil (`limit_minute = X`) l'activation stricte et paramétrée sur des routes cibles.
- **Critère d'Acceptation :** Rejet absolu et déni de service ultra-rapide des flux illégitimes (matérialisé par IP) avec déclenchement `HTTP 429 Too Many Requests` court-circuitant le processus dès la lisière du handler métier généré, entraînant l'interdiction sévère d'implication de transactions en base, d'allocations de métadonnées, ou de logs de fond.

</details>

<details>
<summary>[x] 2.2. Pare-feu d'infiltration AOP (JWT & OIDC)</summary>

- **Objectif :** Isoler hermétiquement l'application de toute vérification identitaire manuelle à risque. Empêcher les faiblesses cryptographiques en laissant l'AOP piloter logiciellement cette surface de validation.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`core.rs`) :** Implémenter la primitive de décodage cryptographique asynchrone (OIDC/JWT) native au framework (ex: crate `jsonwebtoken`), exposée globalement pour une vérification mathématique unifiée sans surcharger l'AST généré.
  - **Étape 2 (`handler_generator.rs`, `dao_generator.rs` & `server.rs`) :** Le framework est actuellement coupé des en-têtes HTTP ! Imposer à `dao_generator.rs` d'ajouter `pub headers: lightx::ext::hyper::HeaderMap` et `pub user_id: Option<String>` au `RequestContext`. Mettre à jour `server.rs` pour cloner dynamiquement `req.headers()` vers le contexte. Cela permettra enfin au handler `auth="jwt"` de lire le token d'autorisation, d'appeler l'intercepteur, et d'y stocker la signature sans allocation Heap (remplacement total du `TypeMap`).
- **Critère d'Acceptation :** L'activation stricte de `auth = "jwt"` dans le gabarit `.toml` rend la route radicalement inatteignable (rejet court-circuité `401`) en l'absence de clé de transport licite.

</details>

<details>
<summary>[x] 2.3. Observabilité vectorisée (OpenTelemetry / tracing)</summary>

- **Objectif :** Atteindre l'extrême visibilité globale, vitale aux écosystemes d'industrialisation ou architectures à hautes topologies virtuelles isolées.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`Cargo.toml`) :** Fusionner harmonieusement les implémentations natives Rust de `tracing` et ses canaux OTLP (OpenTelemetry).
  - **Étape 2 (`core_generator.rs`) :** Instaurer une propagation continue du tag `trace_id` depuis la racine (via l'injection statique d'un instrument `tracing::span` par le routeur `route_request`) jusqu'au cœur de la matrice asynchrone de la base.
  - **Étape 3 (`Cargo.toml`) :** Activer impérativement la feature `tracing` du crate `sqlx` pour exposer et croiser nativement le spectre d'alerte des erreurs et durées quantiques des requêtes sans l'empreinte d'un logger intermédiaire manuel.
- **Critère d'Acceptation :** Connexion asynchrone non-pernicieuse aux tableaux de contrôle Jaeger / Prometheus s'exécutant au rythme naturel du flux métier sans intervention manuelle (zéro-log boilerplate).

</details>

<details>
<summary>[x] 2.4. Audit de sécurité indépendant & certification</summary>

- **Objectif :** Valider formellement la robustesse aux normes de l'industrie (oWASP, CVE) avant d'autoriser le sceau "Production-Grade".
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`git`) :** Geler une Release Candidate de base (ex: v1.0.0-rc.1) une fois les phases 0 à 2 achevées.
  - **Étape 2 (`SecOps`) :** Soumettre l'architecture globale (du CLI aux Handlers) à un audit tiers (ou automatisé via outils SAST & DAST) pour tester les vulnérabilités résiduelles aux débordements mémoires, DoS ou asynchronie mal isolée.
- **Critère d'Acceptation :** Rapport de sécurité disponible certifiant le niveau de protection pour un usage en milieu financier ou critique.

</details>

<details>
<summary>[x] 2.5. Bouclier heuristique absolu (Fuzzing & Miri)</summary>

- **Objectif :** Garantir mathématiquement l'invulnérabilité mémoire et l'endurance absolue aux charges et payloads corrompus.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`cargo-fuzz`) :** Configurer un environnement `cargo-fuzz` au sein du workspace. Exposer logiciellement un _harness_ (gabarit) qui noiera spécifiquement le pipeline HTTP et le parseur JSON avec des charges mémoires absurdes, aléatoires et tronquées.
  - **Étape 2 (Code `unsafe`) :** Si la technique de routage en _Zéro-Copie_ requiert tout bloc `unsafe` pour atteindre la vitesse de la lumière (pointer dereferencing), lier par lint une obligation d'un commentaire `// SAFETY:` formel sur la ligne précédente.
  - **Étape 3 (`.github/workflows/ci.yml`) :** Insérer une phase de CI (nightly de préférence) activant `cargo miri test` pour qu'il balaye le module et confirme de manière déterministe l'absence de Comportement Indéfini (UB).
- **Critère d'Acceptation :** Même une attaque délibérée avec désynchronisation binaire ne crée aucune panique ou corruption (Déni de service - DoS désamorcé à zéro).

</details>

---

## Phase 3 : Modèle financier, multibase & performance brutale (FinOps)

<details>
<summary>[x] 3.1. Couche haute vitesse de cache radical</summary>

- **Objectif :** Protéger économiquement les ressources d'infrastructure, shunter purement la base de vérité lors d'accès majeurs récurrents en lecture simple.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`server.rs`, `core_generator.rs` & `dao_generator.rs`) :** L'architecture actuelle bride l'extensibilité car `route_request` (et le Trait `Router`) renvoie stricto sensu un `Result<String, AppError>`, forçant `server.rs` à recréer en dur une réponse `application/json` (via `success_response`). Refactoriser la signature globale de l'AOP et du Trait `Router` pour renvoyer la réponse réseau intégrale `Result<lightx::ext::hyper::Response<lightx::ext::http_body_util::Full<lightx::ext::bytes::Bytes>>, AppError>` (indispensable pour propager les Bytes exacts **ET** d'autres `Content-Type` sans être piégé par `server.rs`). Déclarer ensuite un second registre de cache dédié (ex: pool `moka` asynchrone) dans `server.rs` (au sein de `listen` et `listen_tls`) et forcer `dao_generator.rs` à l'injecter sous le nom `pub response_cache` au sein du `RequestContext`.
  - **Étape 2 (`handlers/*.toml`) :** Accorder au protocole TOML des routeurs la capacité d'injecter une balise directive statique (stratégie préemptable + dimension temporelle d'expiration TTL).
  - **Étape 3 (`handler_generator.rs`) :** Imposer à l'orchestrateur métier (AOP) de lire le `response_cache` depuis le `RequestContext` en pré-condition et, en cas de _Cache Hit_, de retourner directement la `Response` pré-assemblée contenant son empreinte `Bytes`, annihilant instantanément la descente vers le Business Object et la base de données.
- **Critère d'Acceptation :** Le Business Object n'est plus appelé si sa ressource statique est sous cache ; latence de transaction divisée exponentiellement de manière native.

</details>

<details>
<summary>[x] 3.2. Agnosticisme formel multibase (PostgreSQL / SQLite)</summary>

- **Objectif :** Ne plus dépendre d'une matrice logicielle unique. Répondre militairement aux cahiers des charges d'infrastructures décentralisées et d'applications ultra-légères.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`core.rs` & `server.rs`) :** Éliminer radicalement l'idée mortelle d'un trait dynamique global `DatabaseAdapter` (dont la _VTable_ violerait la norme Zéro-Allocation). Pire encore, `server.rs` ne doit plus conserver aucun attribut statique `pool: sqlx::MySqlPool` (une hérésie en cas de multi-bases) : confier la totalité du routage des moteurs DB au `ContextFactory` (défini à l'Étape 0.4.1) qui stockera et instanciera de manière générique les pools requis.
  - **Étape 2 (`dao_generator.rs`) :** Abstraire l'exécution d'introspection et la génération des chaînes littérales de contextes (`sqlx::MySqlPool` vs `sqlx::PgPool`) via la détection ferme des environnements de compilation Cargo (ex: vérification statique `if std::env::var("CARGO_FEATURE_POSTGRES").is_ok()`). Le `RequestContext` généré restera ainsi 100% statique de bout en bout, avec les types SQLx formels et natifs injectés physiquement par le générateur à la compilation.
  - **Étape 3 (`tests`) :** Coder une procédure d'intégration certifiant la compilation et l'exécution sous dialecte PostgreSQL pur, validant mathématiquement le routing natif du dialecte via CI.
- **Critère d'Acceptation :** Le système entier s'articule dynamiquement autour de la base ciblée par modification unique d'une variable Cargo, tout en réussissant les cas de tests spécifiques au SGBD.

</details>

<details>
<summary>[x] 3.3. Radix Trie Absolu & Preuves de Performances (Benchmarks réels)</summary>

- **Objectif :** Ne plus revendiquer la victoire intellectuelle sans données mathématiques brutes ; s'assurer du scale infini sur le rooting.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`core_generator.rs`) :** Démanteler le routeur séquentiel actuellement généré (qui sépare le path via `split('/').collect()` imposant une mortelle allocation native sur le Heap d'un `Vec<&str>`). Basculer sur une structure _Radix Tree_ (type `matchit`) prouvant algorithmiquement un parsing sans allocation mémoire et un temps de résolution temporel absolu en _O(k)_.
  - **Étape 2 (`tests` & `lightx-benchmark`) :** Publier un sous-crate de test confrontant systématiquement le framework contre de l'industriel (`Axum`, `Salvo`) via `criterion` et soumis à charge massique via `wrk` / `bombardier`.
  - **Étape 3 (Global) :** Construire physiquement l'intégration logicielle aux normes des **TechEmpower Web Framework Benchmarks** en préparant une PR officielle à envoyer sur leur dépôt central.
- **Critère d'Acceptation :** Éligibilité validée au classement officiel TechEmpower mondial (TEB), le framework certifiant mathématiquement faire partie du TOP mondial de l'écosystème Rust.

</details>

---

## Phase 4 : Interface web temps-réel et frontend serveur

<details>
<summary>[x] 4.1. Template engine natif "zero-overhead" & HTML / HTMX</summary>

- **Objectif :** Offrir à l'humain la possibilité d'architecturer des Interfaces UI sans sacrifier aux dépendances monumentales de frameworks applicatifs type React.
- **Spécifications (à ne pas interpréter) :**
  - **Étape 1 (`Cargo.toml` & `handler_generator.rs`) :** Intégrer formellement le moteur maison `tmplx` garantissant l'absence de parsing au runtime. Imposer à `handler_generator.rs` de capter les routes marquées `type = "html"` dans le TOML, de s'abstenir de tout JSON, et de générer l'appel statique `.render()` sur le DTO rendu par les Business Objects.
  - **Étape 2 (`handler_generator.rs`) :** Fournir le contrat de restitution final sous forme de structures générées qui transposeront le rendu du moteur HTML en construisant et relâchant la `hyper::Response` complète (Phase 3.1) dotée obligatoirement de son header absolu `Content-Type: text/html; charset=utf-8`.
- **Critère d'Acceptation :** Empêcher irrémédiablement la compilation d'un fichier source si le développeur renvoie en Rust un composant ou un nom de valeur inexistante pour sa page HTML couplée.

</details>

<details>
<summary>[x] 4.2. Protocole asynchrone persistant (WebSocket déclaratif)</summary>

- **Objectif :** Supporter des tunnels massifs temporels pour garantir l'échange instantané continu des requêtes clientes en temps masqué.
- **Spécifications (à ne pas interpréter) :**
  - [x] **Étape 1 (`server.rs` & `dao_generator.rs`) :** Gérer l'ascension du flux (Upgrade Connection) de `hyper`. **Attention fatale architecturale** : L'architecture actuelle effectue un `.collect()` (Phase 0.2) incompatible avec un flux réseau infini TCP WebSocket. `server.rs` DOIT intercepter l'entête `Upgrade: websocket` et suspendre sa lecture tamponnée. Pour transmettre ce socket au routeur, forcer `dao_generator.rs` à générer le champ dédié `pub raw_req: Option<lightx::ext::hyper::Request<lightx::ext::hyper::body::Incoming>>` dans le `RequestContext`. `server.rs` y injectera la requête non consumée.
  - [x] **Étape 2 (`handler_generator.rs`) :** Pour les routes TOML cibles `type = "websocket"`, générer une boucle évènementielle asynchrone (ex: `tokio-tungstenite`) ancrée dans le handler AOP. Ce pare-feu devra désérialiser et valider la structure JSON de chaque trame WS brute entrante avant de relayer l'objet métier propre au Business Object, assurant un pipeline temps-réel impénétrable.
- **Critère d'Acceptation :** Tolérance parfaite à l'orchestration non-bloquante du canal avec certitude formelle des Payload JSON (le flux temps-réel doit intercepter les données corrompues en vol sans faire paniquer le thread TCP de l'interface).

</details>

---

## Phase 5 : Écosystème Industriel & Sécurité Architecturale

<details>
<summary>[x] 5.1. État Partagé et Télédiffusion (Pub/Sub WebSocket)</summary>

- **Objectif :** Élever la couche WebSocket d'un simple mode "Request-Response" asynchrone vers une architecture événementielle serveur autorisant le Push multicanal.
- **Spécifications (à ne pas interpréter) :**
  - [x] **Étape 1 (`core.rs` & `server.rs`) :** Injecter statiquement un `GlobalState` type-safe au sein du `AppContextFactory`, motorisé par un canal MPMC (`tokio::sync::broadcast`) interdisant formellement l'usage de `std::sync::Mutex` (risque d'empoisonnement bloquant) au profit exclusif de Mutex asynchrones limités ou de structures atomiques.
  - [x] **Étape 2 (`handler_generator.rs`) :** Transformer formellement la boucle séquentielle `ws.next().await` en une architecture de multiplexage (via `tokio::select!`). Cette macro écoutera conjointement la socket TCP entrante et le récepteur `rx` du canal MPMC, permettant l'émission instantanée d'un DTO JSON sans suspendre l'écoute cliente.
- **Critère d'Acceptation :** Validation par un test unitaire où une mutation en Base de Données par un Business Object "HTTP POST" provoque la réception instantanée d'un message sur un pool de clients WebSocket connectés sans aucune congestion temporelle.
</details>

<details>
<summary>[x] 5.2. Boucliers Globaux Périphériques (Middlewares)</summary>

- **Objectif :** Protéger le pipeline avant même la résolution de l'URI (Radix Trie) et unifier les règles de distribution pour garantir la scalabilité sous attaques massives (Rate-Limiting, DDOS).
- **Spécifications (à ne pas interpréter) :**
  - [x] **Étape 1 (`server.rs`) :** Architecturer l'implémentation de la directive `middlewares.toml` en générant une couche d'encapsulation native stricte (`tower::Layer`) encapsulant le `Service` Hyper. Cette couche interceptera la `Request<Incoming>` _avant_ toute allocation de contexte (CORS stricts, HSTS Security Headers, IP Blocklist).
  - [x] **Étape 2 (`cargo`) :** Enchérir l'arbre de dépendances avec `tower-http` pour implémenter rigoureusement les couches `CompressionLayer` et `DecompressionLayer`, garantissant la négociation Content-Encoding (GZIP/Brotli) par streaming matériel sans sollicitation de la RAM applicative.
- **Critère d'Acceptation :** Rejet asynchrone d'une requête HTTP `wrk` illégitime sous les 5 micro-secondes avant même son entrée dans le Radix Trie, certifiant la résilience DDOS de la façade.
</details>

<details>
<summary>[x] 5.3. Routage Statique et Delivery (Zero-Copy Assets)</summary>

- **Objectif :** Affranchir l'application du besoin systémique d'un Proxy externe (Nginx) pour délivrer massivement les fichiers statiques sans épuiser la RAM du Node.
- **Spécifications (à ne pas interpréter) :**
  - [x] **Étape 1 (`core_generator.rs`) :** Ajouter une directive de sous-routage TOML `type = "static"` paramétrisant un noeud terminal du Radix Trie pointant vers un service natif interfaçant un flux brut (`tokio_util::io::ReaderStream`). L'objectif est d'interdire le chargement des fichiers en RAM et d'imposer un transfert Chunking _Zero-Copy_.
  - [x] **Étape 2 (`build.rs`) :** Garantir que les en-têtes d'invalidation (Cache-Control, ETag) ne feront l'objet d'aucun calcul d'IO par le Runtime. Ils doivent mathématiquement découler de hashs cryptographiques (SHA-256) calculés exclusivement au moment de la génération (Build Time) par le macro-moteur.
- **Critère d'Acceptation :** L'obtention invariable de la note `A+` sous un audit de performances Lighthouse complet en conditions locales, prouvant la parfaite gestion des ETags et du Cache Control.
</details>

<details>
<summary>[x] 5.4. Exécuteurs Asynchrones Absolus (Background Tasks)</summary>

- **Objectif :** Empêcher irrémédiablement la mort prématurée des processus chronophages indépendants de la requête client (Envois d'emails, Indexation).
- **Spécifications (à ne pas interpréter) :**
  - [x] **Étape 1 (`server.rs`) :** Initialiser formellement un Orchestrateur Singleton basé sur un canal MPSC lors du démarrage du serveur. Ce superviseur hébergera l'ensemble des descripteurs de tâches asynchrones en dehors du cycle d'acceptation TCP.
  - [x] **Étape 2 (`core.rs`) :** Déclarer la macro de soumission `DeferredTask::spawn(...)`, obligeant le développeur à s'affranchir du `RequestContext` Http et lui accordant un tout nouveau cycle transactionnel dédié de type "Fire and Forget".
- **Critère d'Acceptation :** Enregistrement et exécution intègre prouvée d'une tâche de 5 secondes, alors même que la réponse de la requête HTTP originelle ayant ordonné cette tâche a été envoyée et le flux TCP fermé en moins de 10ms.

</details>

<details>
<summary>[x] 5.5. Harness Pédagogique (SuperTest Mocking API)</summary>

- **Objectif :** Conférer au développeur le pouvoir de valider logiquement tout l'AppRouter via les tests standards Cargo de façon atomique et déconnectée (Sans TCP Socket).
- **Spécifications (à ne pas interpréter) :**
  - [x] **Étape 1 (`lightx::test`) :** Designer une API Builder matricielle (inspirée du concept `SuperTest` JS/TS) permettant d'usurper physiquement la création d'une structure `hyper::Request<Incoming>` en RAM.
  - [x] **Étape 2 (`core_generator.rs`) :** Obliger formellement l'AppRouter généré (implémentant `tower::Service`) à exposer le trait `tower::ServiceExt::oneshot`. Ceci garantira l'appel d'un mock HTTP électriquement sans allocation d'un Listen Port.
- **Critère d'Acceptation :** Les intégrateurs métier (Business Objects), via `cargo test`, manipulent virtuellement des headers, tokens JWT et frames JSON sans jamais devoir dépendre de `tokio::net::TcpListener`.

</details>

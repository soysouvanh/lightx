# API LightX - Guide de Démarrage et Architecture

Bienvenue dans l'API de démonstration du framework LightX. Ce projet sert de vitrine technologique et pédagogique pour comprendre comment utiliser et interagir avec les incroyables capacités du moteur de génération automatique de code propre à **LightX** et **Daox**.

[English](README.md) | [Français](README.fr.md)

---

## 🚀 Démarrage Rapide (Tutoriel Pas à Pas)

Ce guide est conçu pour être accessible à tous, même aux débutants. Suivez chaque étape minutieusement.

### Étape 1 : Préparer l'environnement de développement

Pour faire tourner ce projet, vous avez besoin des outils de base du langage Rust.

- **Installer Rust** : Si ce n'est pas encore fait, installez le compilateur en ouvrant un terminal et en exécutant la commande officielle :
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  _(⚠️ Indispensable pour les débutants : Une fois l'installation terminée, redémarrez votre terminal ou exécutez `source $HOME/.cargo/env` pour activer la commande `cargo`)._

### Étape 2 : Préparer les Bases de Données

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

### Étape 3 : Démarrer le Serveur LightX

Maintenant que Rust est installé et que les bases de données tournent, vous pouvez démarrer l'API.

1. **Placez-vous dans le répertoire du projet `lightx-api` :**

   ```bash
   cd lightx-api
   ```

2. **Lancez le compilateur en développement :**

   ```bash
   cargo run
   ```

   > 💡 **Que se passe-t-il ici ?** Lors de ce démarrage automatisé, le framework va analyser tous vos modèles TOML, générer l'entièreté des requêtes SQL (DAO), orchestrer les routeurs (AOP), compiler le tout, puis lancer le puissant serveur asynchrone sécurisé sur le port `3000`.

   > ✅ Vous saurez que tout fonctionne parfaitement lorsque vous verrez apparaitre le message : `Démarrage de LightX API (JSON REST)!`. (Le serveur bloquera ce terminal, c'est normal, il attend vos requêtes).

### Étape 4 : Tester les Points de Terminaison (Endpoints)

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

---

## 🧠 Comprendre le Framework : Excellence et Architecture

Ce projet ne se contente pas des traditionnelles API REST. Il vise à inculquer les pratiques de l'**Excellence Pédagogique et Technologique (SOTA)** adoptée par LightX.
Voici tout ce qu'il vous faut pour démystifier le rôle des différentes couches et développer en parfaite sécurité.

### 1. La Puissance des Macros (Zéro Surcout)

LightX prohibe formellement la _réflexion informatique classique_ (l'introspection lente pendant l'exécution qu'utilisent beaucoup de frameworks).
Au moment précis où vous exécutez `cargo run`, le moteur lit vos `TOML` et `SQL` pour forger à la volée du **pur code de bas niveau Rust** hyper-optimisé (via _Daox_).

- **Conséquence en Production** : Routage foudroyant (Vitesse `O(1)`), sécurité mémoire absolue (absence de fuites imprévues) et un code strictement immunisé natif face aux vulnérabilités d'injection SQL.

### 2. La Magie de L'Orienté Aspect (AOP)

Dans ce dépôt, vous remarquez le sous-dossier `handlers/` et ses fichiers TOML. C'est eux qui commandent l'AOP.
Le framework génèrera un super-contrôleur imperméable à toute faille qui se charge :

- D'analyser le JSON entrant et l'URL selon les règles strictes.
- D'assurer l'aiguillage le plus direct vers la bonne fonction.
- D'exécuter l'enchainement de vos **Objects Métiers / Business Objects (BO)**.
- Option vitale : De décider souverainement s'il émet un `COMMIT` ou s'il interrompt tout sur un `ROLLBACK` via SQL (en cas d'erreur minime sur n'importe quel traitement).

### 3. Écosystème des Couches (Séparation Rigueur)

Chaque brique a une responsabilité immuable, conçue pour vous faciliter la vie (Accessibilité) ;

1. **La Couche d'Accès aux Données (DAO)** : Générée par nos librairies (Daox). Elle exécute le gros-œuvre invisible et vous offre des fonctions prêtes à l'emploi (Insert, UpSert, Cursors stream...).
2. **La Couche Métier (BO)** : Située dans le dossier `src/bo/`. **C'est précisément ici que vous écrivez votre code !** Affranchissez-vous du réseau et des requêtes pures en appelant directement le DAO pour implémenter tranquillement vos algorithmes métier.
3. **Le RequestContext (Le Bus)** : LightX n'utilise jamais d'état global risqué. La Context Factory rassemble vos `.env` (ex: `ctx.postgres_pool`) en un environnement paresseux ultra léger, livré directement à vos méthodes métiers (BO) et supprimé magistralement de la mémoire via les règles temporelles Rust (RAII) dès la fin de requête HTTP.

### 4. Zero Panic, 100% Rust Sécuritaire

Sur l'architecture LightX, vous allez interagir au quotidien avec l'erreur `AppError` visible (par exemple dans le fichier métier expérimental `DbDemoBo.rs`).
L'intention globale bannit rigoureusement l'usage inattendu des crasheurs tels que `panic!()` ou `unwrap()`.

**Le mécanisme SOTA (State of the Art)** : Si une boucle métier panique, qu'un SQL renvoie une ligne manquante, l'erreur pure est captée de plein vol. L'énumérateur `AppError` va traduire en langage universel l'exception via `?`, recracher une formidable réponse JSON structurée au Frontend, et votre serveur REST tiendra sa promesse : continuer à servir la population des autres clients sans jamais arrêter l'exécutable natif !

---

**Félicitations**, vous commencez l'aventure avec les certitudes et les secrets internes de la machinerie universelle et foudroyante de LightX !

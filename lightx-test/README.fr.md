# Application de Test LightX (Starter Kit)

Bienvenue dans l'**Espace de jeu LightX**. Ce projet est le bac à sable idéal pour permettre à n'importe quel développeur de s'approprier et maîtriser la puissance du framework LightX.

À travers ce dépôt, vous allez expérimenter la magie de l'introspection **"Database-First"**, de la Programmation Orientée Aspect (AOP) appliquée au Cœur HTTP, et de la génération **"Shift-Left"** (où toutes les failles de sécurité et anomalies de type sont pulvérisées au moment de la compilation plutôt qu'à l'exécution).

---

## Cartographie : Par où commencer ?

LightX impose une architecture extrêmement robuste et disciplinée. Chaque dossier remplit une, et une seule fonction critique.
_Nous vous recommandons chaleureusement de rentrer dans chaque dossier pour en lire le `README.fr.md` dédié, écrit spécialement pour les novices !_

- **`schema/`** : Le cerveau. Contient le calque `TOML` de votre base SQL, généré automatiquement. **[Ne modifiez jamais ces fichiers]**
- **`overrides/`** : L'exception humaine. C'est ici que vous redéfinissez des règles métiers sur vos colonnes (ex: longueur minimale) ou que vous créez des **Paramètres Virtuels** (`accept_terms`).
- **`handlers/`** : La logique HTTP. Vous déclarez ici vos API en `TOML`, que LightX métamorphose en routeur natif Rust de très haute performance.
- **`i18n/`** : Les langues. Consigne et cartographie les erreurs (ex: "Texte trop court") pour renvoyer un superbe JSON normalisé au Frontend.
- **`log/`** : La boîte noire. Où LightX enferme silencieusement les traces de crash back-end pour protéger votre serveur (concept "Panic-Free").

---

## Cycle de Vie d'une Requête (Fonctionnement Pédagogique)

Pour bien démarrer avec le Starter Kit, décomposons la magie de LightX lors de l'exécution d'une requête HTTP :

### Les Couches Générées vs Manuelles

- **Ce qui est généré (La Machine) :** Le **Router (Core)** (qui décode l'URL HTTP), le validateur **`check_parameters`** (qui s'assure de l'intégrité de la donnée via la base SQL et cast les types), le **Handler (AOP)** (qui orchestre les appels métiers fail-fast), et le **DAO** (qui exécute les requêtes SQL fortement typées).
- **Ce qui est manuel (L'Humain) :** Les **Fichiers de Configuration (`.toml`)** et les **Business Objects (BO)**, où vous écrivez l'intelligence de votre application (vos règles métiers).

### Le Flux Détaillé de Bout en Bout (End-to-End)

1. **La Requête HTTP :** Le client envoie un `POST /api/users`.
2. **Le Router (Core) & Validation :** L'aiguilleur ultra-rapide trouve le bon chemin en `O(1)`. Il lance immédiatement `check_parameters` pour vérifier la charge utile (ex: "l'email fait-il plus de 5 caractères ? est-il bien formaté ?"). Si c'est invalide, il renvoie instantanément une erreur `HTTP 400 JSON`. Pas un seul appel n'est fait à la base de données.
3. **Le Handler (AOP) :** Votre requête est saine. L'orchestrateur généré déroule l'exécution des _Business Objects (BO)* via une phase de validation (Fail-Fast) puis une phase de traitement (transactionnelle).
4. **Le BO (Business Object) :** La seule fonction que VOUS avez écrite en Rust. Elle prend la donnée pure (et déjà typée en toute sécurité), applique votre algorithme métier (ex: vérification de solde métier), et sollicite au besoin la base locale.
5. **Le DAO :** Si votre BO a besoin d'écrire en BDD, le DAO ouvre paresseusement (lazy load) la transaction réseau SQL et interroge la base via vos requêtes strictement vérifiées à la compilation.
6. **La Réponse :** Le framework valide la transaction automatiquement, et sérialise de manière gracieuse les résultats dans un flux `HTTP 200 OK` (JSON ou HTML). S'il y a eu la moindre erreur native en cascade, le destructeur RAII gère le Rollback automatiquement en base.

### Le Rôle du Développeur

Pour créer une nouvelle API fonctionnelle de A à Z avec LightX, vous n'aurez que **4 étapes simples** à suivre :
<br>

<div align="center">
  <img src="../assets/dev_workflow.svg" alt="Workflow du Développeur" width="90%">
</div>
<br>

1. **La Base (SQL) :** Créer la table dans votre base de données (ex: table `users`). Le générateur introspectera tout le reste (création des TOML primitifs dans `/schema`).
2. **Surcharges & Paramètres Virtuels (TOML) :** Si la BDD pilote tout, comment gérer des données purement web ? Vous créez manuellement des fichiers `.toml` dans le dossier `overrides/` pour _surcharger_ une règle existante de BDD (ex: imposer `min_length = 8` au mot de passe) ou créer de toutes pièces des **Paramètres Virtuels** (ex: `password_confirmation` ou `accept_terms` qui n'existent pas en base).
3. **La Route (TOML) :** Déclarer votre Handler au sein de `handlers/RegisterUser.toml` en choisissant les champs qu'il accepte (BDD et Virtuels), et l'ordre des `BO` à appeler.
4. **Le Métier (Rust) :** Dans le dossier de vos BO (ex: `src/bo/user_bo.rs`), créer une fonction asynchrone prenant l'unique objet conteneur `&mut RequestContext`, écrire votre algorithme pur, et appeler votre accès final `UserDao::insert...`.

---

## Étape 1 : Configuration de la Base de Données (La Source de Vérité)

LightX place votre base de données réelle comme concept central !

1. Connectez-vous à votre serveur MySQL (version 8.0+ recommandée) et créez une base vierge nommée `lightx_test` :
   ```bash
   mysql -u root -p -e "DROP DATABASE IF EXISTS lightx_test; CREATE DATABASE lightx_test CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;"
   ```
2. **Initialisation des Modèles & Données de Test :**
   ```bash
   cargo run -p lightx-cli -- migrate
   ```

## Étape 2 : L'Environnement

Pour que la génération ait lieu, le framework doit connaitre vos identifiants.
Créez un fichier `.env` à la racine de `lightx-test` (ou copiez `.env.example`) :

```env
DATABASE_URL=mysql://root:password@localhost:3306/lightx_test
LIGHTX_LOG_DIR=./log
```

> *( N'oubliez pas de remplacer `root` et `password` par vos véritables identifiants de développement local)._

---

## Étape 3 : Compilation Shift-Left & Génération

Tout est prêt ! Démarrez l'industrie de méta-programmation avec une commande symbolique :

```bash
cargo build
```

### Que fait `cargo build` en secret ?

Votre fichier `build.rs` agit comme un orchestre DevOps 100% autonome :

1. **Introspection (DaoGenerator)** : Navigue dans votre BDD et modélise mathématiquement les tables sous forme `.toml` dans le dossier `schema/`.
2. **Fabrication AOP (HandlerGenerator)** : Lit vos endpoints dans `handlers/` et code littéralement un routeur Rust ultra-rapide (en `O(1)`) qui servira de mur pare-feu contre les mauvaises données entrantes.
3. **Câblage (CoreGenerator)** : Prépare le serveur HTTPS final.
4. **Validation SQL** : Le compilateur interroge nativement le serveur MySQL pour s'assurer que le code généré ne contiendra aucune requête SQL cassée.

### Où se trouve ce fameux code généré ?

Pour conserver votre arborescence parfaitement propre et claire pour l'esprit (Zéro-Overhead), **AUCUN code généré n'est recraché dans `src/`**.
Toutes les structures natives sont injectées directement dans les abysses du compilateur (dans `target/debug/build/`). Vous ne voyez que ce qui est utile !

## Étape 4 : Démarrage Moteur

Vous pouvez maintenant démarrer l'application et assister au spectacle !

```bash
cargo run
```

Vous verrez la console LightX traverser avec succès le test des DAOs, l'orchestration des données (BO), l'exécution simulée d'une requête HTTP (Handler), et enfin allumer son grand serveur HTTPS embarqué !

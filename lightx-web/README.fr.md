# LightX Web - Le Dashboard Full-Stack (Tmplx)

Bienvenue dans l'interface Web pédagogique du framework LightX. Ce projet sert de vitrine d'architecture Front-end & Back-end, démontrant comment utiliser et interagir de manière exhaustive avec les exceptionnelles capacités de rendu du moteur **Tmplx** (Zero-Allocation) couplé à l'AOP de **LightX** et la génération **Daox**.

[English](README.md) | [Français](README.fr.md)

---

##  Démarrage rapide (tutoriel pas à pas)

Ce guide est conçu pour vous montrer toute l'étendue de l'architecture SSR (Server-Side Rendering) avec des performances State-of-the-Art.

### Étape 1 : Préparer l'environnement de développement

Pour faire tourner ce projet, assurez-vous d'avoir Rust :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Étape 2 : Lancer l'application (Magie du Build)

L'avantage absolu de cette vitrine est qu'elle utilise **SQLite en mémoire vive (sqlite::memory:)** et les structures _offline_ générées par Daox. Absolument rien n'est à installer.

1. **Placez-vous dans le répertoire du projet `lightx-web` :**

   ```bash
   cd lightx-web
   ```

2. **Lancez le compilateur en développement :**
   ```bash
   cargo run
   ```

>  **Que se passe-t-il ici ?** Lors de la commande, le framework va :
>
> 1. Invoquer le compilateur **Tmplx** : Ce dernier lit le dossier `templates/`, vérifie mathématiquement la syntaxe, et convertit tout votre code HTML en macros Rust pures à vitesse `O(1)` sans aucune allocation dynamique (0 bytes de heap !).
> 2. Analyser les modèles et orchestrer l'AOP de vos routeurs `handlers/`.
> 3. Lancer le puissant serveur web asynchrone sur le port `8081`.

### Étape 3 : Découvrir le Dashboard Exhausif (Tmplx)

Félicitations, le serveur tourne. Allez sur votre navigateur et ouvrez `http://localhost:8081/dashboard`.

Vous découvrirez un tableau de bord (Dashboard) ultra-puissant qui démontre la maîtrise complète des variables et instructions du moteur **Tmplx** :

- **Variables et Échappements (Raw/Secure)** : `{%%= view_data.message %}` vs `{%= view_data.id %}`
- **Logique Algorithmique (Boucles et Conditions)** : `{% for user in view_data.users %}` et `{% if user.is_admin %}`
- **Architecture Modulaire HTML** : Le Layout racine via `extends`, les blocs dynamiques via `block` et les modèles partiels réutilisables via `include`.
- **Méthodologie CRUD API** : Les pages View, Add, et Edit s'enchaînent instantanément pour prouver que vous pouvez mixer l'AOP LightX (Handlers) sans aucune fatigue rédactionnelle !

---

##  Architecture et fondations du SSR Rust

### L'approche "Duck-Typing" du Tmplx Compiler

Où réside l'exploit de `lightx-web` par rapport au reste de l'écosystème Rust web ? Dans l'obligation des modèles de vues !
Ici, si vous renommez la propriété `pseudo` en `name` dans votre Business Object `dashboard.rs`, **la compilation de vos pages HTML s'arrête en erreur instantanément !**

Grâce à `Tmplx` inclu via votre `build.rs`, votre HTML n'est pas un texte perdu. Il devient un AST (Arbre Syntaxique) et demande au compilateur `rustc` de certifier que _toutes les variables invoquées_ dans le DOM existent sur les structures renvoyées par le BO avant d'accepter le déploiement. C'est l'assurance qualité absolu.

Bon développement sur LightX !

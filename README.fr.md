[English](README.md) | [Français](README.fr.md)

# Espace de travail LightX

![Version](https://img.shields.io/badge/version-v1.0.0--beta-blue)
![Architecture](https://img.shields.io/badge/architecture-Database--First-success)
![Routing](<https://img.shields.io/badge/Router-O(1)-orange>)
![Security](https://img.shields.io/badge/Security-TLS%2FHTTPS-purple)

**LightX** est un framework Rust ultra-optimisé conçu pour la mise en production d'entreprise. Il repose sur une approche **Database-First** stricte et une philosophie **Zéro-Overhead** (aucune perte de performance logicielle).

Au lieu de rédiger du code fastidieux, LightX inspecte silencieusement votre base de données MySQL et déduit toute l'architecture de votre serveur backend (Modèles métiers, Routeurs réseau, Pare-feu de validation) directement au moment de la compilation.

---

## Architecture Globale (Workspace)

Voici comment s'organise l'écosystème LightX au travers de ce Cargo workspace :

```text
lightx-workspace/
├── lightx/                 # Le Moteur Technique (macros, générateurs, serveur HTTPS)
│   ├── src/                # Logique réseau, bus de données, et validation
│   └── assets/             # Diagrammes SVG de l'architecture technique
│
├── lightx-test/            # Le Starter Kit vitrine (Documentation vivante)
│   ├── handlers/           # Fichiers d'orchestration TOML purs
│   ├── migrations/         # Schémas SQL de la base de données
│   └── src/                # DAOs auto-générés & Business Objects métiers
```

_Note : Si vous êtes un développeur novice cherchant à apprendre à coder sa propre application avec le framework LightX, rendez-vous immédiatement dans `lightx-test` !_

---

## Les 3 Piliers du Framework (Rappel)

Pour comprendre LightX, il suffit d'intégrer le rôle de ses 3 différentes couches :

<div align="center">
  <img src="./assets/architecture.svg" alt="Architecture Core LightX" width="80%">
</div>

### 1. DAO (Data Access Object)

Généré algorithmiquement en analysant votre BDD SQL. Il fabrique des structures Rust 100% fiables, et ce de manière furtive au fond de la mémoire du compilateur. Fini la pollution du dépôt avec du code redondant, et fini les requêtes SQL cassées !

### 2. AOP (Programmation Orientée Aspect)

Vos chemins d'API sont déclarés via de simples fichiers `.toml` épurés (`handlers/AdminCreation.toml` par ex). LightX les lit en amont et invente un routeur "Pare-feu" mathématiquement infaillible (`Fail-Fast`) pour bloquer toute la data malveillante entrante au coût minimum `O(1)`.

### 3. BO (Business Object)

C'est le bunker. Le lieu de travail de vos ingénieurs. Solidement protégé par le pare-feu AOP, votre code métier ne réceptionnera et ne manipulera que des données irréprochables et typées avec fermeté.

---

## Gestion des Erreurs Infaillible (Panic-Free)

<div align="center">
  <img src="./assets/failfast.svg" alt="Propagation Fail-Fast" width="80%">
</div>

LightX est conçu pour repousser les crashs. Les traditionnels `unwrap()` et `panic!()` de Rust ont été éradiqués de toute la logique générée.
Qu'il s'agisse d'un format erroné (`400 Bad Request`), d'un chemin introuvable (`404`), ou d'une règle métier violée (`422`), LightX transforme silencieusement l'anomalie pour livrer à votre client ou votre front-end un beau paquet JSON :

```json
{
  "code": 422,
  "field": "email",
  "error": "Seuls les domaines @lightx.com sont autorisés."
}
```

## Cycle de Vie d'une Requête (Fonctionnement Pédagogique)

Pour comprendre LightX, il faut distinguer la magie automatique du code réel écrit par le développeur.

### Les Couches Générées vs Manuelles

- **Ce qui est généré (Machine) :** Le **Router (Core)** (qui parse l'URL http), le validateur **`check_parameters`** (qui s'assure de l'intégrité de la donnée via votre BDD et cast les types), l'**Handler (AOP)** (qui orchestre les appels métiers fail-fast), et le **DAO** (qui exécute les requêtes SQL paramétrées).
- **Ce qui est manuel (Humain) :** Les **Fichiers de Configuration (`.toml`)** et les **Business Objects (BO)**, où vous écrivez l'intelligence de votre application (vos règles métiers).

### Le Flux Détaillé de Bout en Bout (End-to-End)

1. **La Requête HTTP :** Le client envoie un `POST /api/users`.
2. **Le Router (Core) & Validation :** L'aiguilleur ultra-rapide trouve le bon chemin en `O(1)`. Il lance immédiatement `check_parameters` pour vérifier la charge utile (ex: "l'email fait-il plus de 5 caractères ? est-il bien formaté ?"). Si c'est invalide, il renvoie instantanément une erreur `HTTP 400 JSON`. Rien ne se passe en BDD.
3. **Le Handler (AOP) :** Votre requête est saine. Le Handler orchestre alors séquentiellement l'exécution des _Business Objects (BO)_ selon une phase de validation (Fail-Fast) puis une phase transactionnelle.
4. **Le BO (Business Object) :** La seule fonction que VOUS avez écrite. Elle prend la donnée pure (et déjà typée en toute sécurité), applique votre algorithme (ex: calcul de taxe, vérification de solde métier), et décide d'appeler les objets de données.
5. **Le DAO :** Si votre BO a besoin de la BDD, le DAO ouvre _paresseusement_ (lazy load) la transaction réseau SQL et interroge la base via vos requêtes strictement vérifiées à la compilation.
6. **La Réponse :** Le framework valide la transaction automatiquement, et sérialise de manière gracieuse les résultats dans un flux `HTTP 200 OK` (JSON ou HTML). S'il y a eu la moindre erreur native en cascade, le destructeur RAII gère le Rollback automatiquement en base.

### Que doit faire le développeur ?

Pour créer une nouvelle API fonctionnelle de A à Z avec LightX, un développeur n'a que **4 étapes simples** à suivre :

<br>
<div align="center">
  <img src="./assets/dev_workflow.svg" alt="Workflow du Développeur" width="90%">
</div>
<br>

1. **La Base (SQL) :** Créer la table dans sa base de données (ex: table `users`). Le générateur introspectera tout le reste (création des TOML primitifs dans `/schema`).
2. **Surcharges & Paramètres Virtuels (TOML) :** Si la BDD pilote tout, comment gérer des données purement web ? Le développeur crée manuellement des fichiers `.toml` dans le dossier `overrides/` pour _surcharger_ une règle existante de BDD (ex: imposer `min_length = 8` au mot de passe) ou créer de toutes pièces des **Paramètres Virtuels** (ex: `password_confirmation` ou `accept_terms` qui n'existent pas en base).
3. **La Route (TOML) :** Déclarer son Handler métier au sein de `handlers/RegisterUser.toml` en choisissant les champs qu'il accepte (BDD et Virtuels), et l'ordre des `BO` à appeler.
4. **Le Métier (Rust) :** Créer un simple fichier dans le dossier de ses BO (ex: `src/bo/user_bo.rs`), avec une fonction asynchrone prenant l'unique objet conteneur `&mut RequestContext`, écrire son algorithme pur, et appeler son accès final `UserDao::insert...`. Éxecutez `cargo run` !

## Compiler tout l'écosystème

Si vous avez vraiment besoin de compiler simultanément le moteur LightX ET son projet de test, il vous suffit de taper la commande maîtresse suivante depuis ce grand dossier racine :

```bash
cargo build
```

#  Internationalisation LightX (i18n)

Bienvenue dans le répertoire des traductions sans surcoût d'exécution (Zero-Overhead) !

Ce répertoire est l'ancre stockant les canalisations de vos messages d'erreurs en format TOML pur, toutes strictement couplées avec les contraintes d'intégrité de la base ou l'implémentation métier.

##  Résistance à l'Hyper-Croissance (Scalabilité)
Pour garantir une stabilité absolue et éviter les conflits Git infernaux sur des projets de milliers de tables, cet espace singe farouchement la topologie granulaire de la base de données.

- `schema.toml`: Le modèle de traduction global pour toute erreur générique liée aux types.
- `handlers/`: Lexique défini par les développeurs pour transcrire des ruptures logiques complexes.
- `overrides/`: Miroir exact identifiant les règles métiers sur-mesure pour être injectées au client.

L'objectif ultime est le déport absolu : le moteur rust du serveur Backend passe son temps à manipuler de la clé statique en `O(1)`. Libre ensuite au Frontend d'enrichir le côté visuel (React, Vue, ou autre).
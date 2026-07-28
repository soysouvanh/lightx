#  Paramètres Virtuels (Données hors base de données)

Bienvenue dans le répertoire des **Paramètres Virtuels** !

**Mais qu'est-ce qu'un "Paramètre Virtuel" ?**
Dans LightX, la plupart des données envoyées dans une requête HTTP correspondent directement à une colonne de votre base de données (ex: `users.first_name`). Pour celles-ci, LightX génère automatiquement les règles de validation en introspectant votre BDD.

Cependant, votre API API HTTP aura souvent besoin de données qui **N'IRONT PAS** en base de données ! Par exemple :

- `accept_terms` (Une case à cocher pour accepter les CGU)
- `new_password` (Un mot de passe en clair envoyé pour être crypté durant le traitement métier)
- `captcha_token` (Un jeton temporaire antispam de validation)

Puisque ces champs n'existent pas dans le modèle de la base de données, LightX les appelle des **Paramètres Virtuels**.

##  Comment créer un Paramètre Virtuel ?

Pour déclarer un nouveau paramètre virtuel, il vous suffit de créer un nouveau fichier `.toml` dans ce répertoire `virtual/`.

Par exemple, pour créer et valider rigoureusement une donnée `accept_terms`, créez un fichier `accept_terms.toml` ici avec son schéma de validation strict :

```toml
[type]
value = "bool"
message = "schema.type.message"

[is_optional]
value = false
message = "schema.is_optional.message" # Message retourné à l'API si le paramètre est absent

#  ASTUCE PRO : Les blocs "Enum Values" !
# Si votre paramètre n'accepte qu'une courte liste d'options strictes (ex: couleur ou boolean), utilisez `[enum_values]`.
# LightX va générer un matching d'exécution natif et ultra-rapide (O(1)) et zappera intelligemment toutes les autres lourdes vérifications au build !
[enum_values]
value = ["true", "false"]
message = "schema.enum_values.message"

[business_rules]
must_be_true = "overrides.virtual.accept_terms.business_rules.must_be_true"
```

##  Pourquoi les définir ici ?

L'approche de LightX est orientée **Performance (Zero-Overhead)** et **Fiabilité**.
Plutôt que d'attendre l'exécution de votre fonction technique pour valider les paramètres (au risque de crasher à l'exécution), LightX valide toutes les charges utiles HTTP **"Fail-Fast"** via l'AOP (Aspect-Oriented Programming).
En mettant votre schéma ici, LightX pourra le compiler en un check ultra-robuste qui protègera automatiquement vos points de terminaison HTTP de requêtes frauduleuses ou mal structurées !

##  Comment les lier à mon Routeur (Handler) ?

Une fois votre fichier `.toml` créé ici, ouvrez n'importe quel fichier de route `.toml` situé dans `handlers/` (la où sont définies vos APIs), et ciblez-le en spécifiant une chaine SQL **vide** `""` !

```toml
# Dans votre fichier handlers/MyHandler.toml par exemple
[parameters]
# La chaine vide "" indique à LightX qu'il s'agit d'un paramètre virtuel !
accept_terms = ""
```

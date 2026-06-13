= Contenu
== Configuration du système
// TODO: Peut-être séprarer les aspects vraiment très haut niveau des aspects architecturaux?
- On va configurer le système de manière déclarative
- Utilisation de fichiers de configuration
- Semblable à Kubernetes, Docker Compose, et Terraform
- Avantages:
    - Plus simple de comprendre le système
    - Simplifer les approches GitOps; la config décrit TOUT le système, une
        configuration peut être prise de manière indépendante de avant ou après.
        C'est pas comme une approche impérative ou il faut aussi "annuler" les
        commandes précédentes
    - Globalement c'est beaucoup plus simple a l'utilisation
- Désavantages:
    - Plus compliqué a implémenter vu qu'on déplace toute la complexité dans le systpme au lieu de laisser l'utilisateur se débrouiller
    - Dans un sens, moins flexible pour l'utilisateur
- Découpage de la configuration en resources
    - iface réseau, disque, route, etc.
    - facteur de différenciaction entre les type resources: a expliquer pck je sais pas comment
    - chaque type de resource est géré par uniquement un gestionnaire ("manager") dans lequel est implmenté la logique
- Il y a deux grande catégories de resources:
    - d'une part les "user config", ça c'est la configuration fournise par l'utilisateur, elle est immuable et est la source de toutes les autres
    - d'autre part il y a les "dynamic resources", ces resources découlent directement d'un user config, elle représente une resource réels et peuvent être modifié par le système. Leur utilité est de représenter un systpme qui, par nature, est dynamique. Par exemple une configuration utilisateur DHCP, va devoir configurer une interface de manière largement similaire a une assignation statique par exemple
    - ça permet aussi d'avoir un lien de dépendance orchestré en interne. par exemple lorsqu'on déclare un conteneur, il y a enfaite deux resources: le conteneur et son image. le conteneur dépend de l'image mais c'est implicite, et le téléchargement de l'image peut être séparer de la gestion du conteneur (déjà parce qu'on pourrait directement crée le conteneur sur une archive tar.gz au lieu d'une image remote)
- Maintenant il y a plusieurs problèmatiques, d'une part qu'est-ce qui lance une réconciliation? On a l'upload d'une nouvelle config utilisateur, la réaction a un évenemetn d'une resource (contenenur exit, iface disconnect, disk hotplug, etc.), mais aussi la création d'une sous-resoiurce.
- A partir de la, deux options: soit c'est le dépôt de configuration qui orchestre le tout, soit c'est le manager responsable d'une resource qui s'occupe lui même de lancer la réconciliation. Ces deux approches sont symétriquement opposés, voici le TLDR en anglais

#let row-label(body) = {
    set par(justify: false)

    body
}

#let row(criterion: [], relevance: "", favors: [], rationale: []) = {
    (
        row-label(criterion),
        row-label(relevance),
        row-label(favors),
        rationale,
    )
}

#table(
    columns: (12em, auto, auto, 1fr),
    row-gutter: (2.2pt, auto),
    table.header[Criterion][Relevance][Favors][Rationale],
    ..row(
        criterion: [Global scheduling control],
        relevance: [MEDIUM],
        favors: [Centralized],
        rationale: [A single loop can optimize the reconciliation by taking into
            account child and dependencies.],
    ),
    ..row(
        criterion: [Scheduling flexibility],
        relevance: [MEDIUM],
        favors: [Decentralized],
        rationale: [Each reconciler can tune its own tick interval, backoff, and
            concurrency without coordinating with a central scheduler.],
    ),
    ..row(
        criterion: [Reacting to internal state events],
        relevance: [MEDIUM],
        favors: [Centralized],
        rationale: [Event-driven reconciliation (trigger on state change rather
            than timer) is straightforward to add to a central loop;
            decentralized reconcilers would each need independent subscription
        ],
    ),
    ..row(
        criterion: [Reaction to external events],
        relevance: [MEDIUM],
        favors: [Decentralized],
        rationale: [Events originating outside the state manager (e.g. a
            container crash) can be handled directly by the owning reconciler.
            In a centralized model, all external signals must be funnelled
            through the state manager
        ],
    ),
    ..row(
        criterion: [Stuck reconciler detection],
        relevance: [LOW],
        favors: [Centralized],
        rationale: [Because the central scheduler expects a response from
            `reconcile()`, it can detect a stuck reconciler via timeout.
            Decentralized reconcilers fail silently. Minor concern given the
            small resource count.],
    ),
    ..row(
        criterion: [Automated sub-resource ownership],
        relevance: [LOW],
        favors: [Centralized],
        rationale: [Because the `reconcile()` call also returns the creation
            requests, it is trivial to associate a parent/child relationship.],
    ),
    ..row(
        criterion: [API call overhead],
        relevance: [N/A],
        favors: [Centralized],
        rationale: [`reconcile(Resource)` bundles all necessary state into one
            call vs. multiple fetches per tick in the decentralized model. Not a
            factor given no API rate-limit constraints.],
    ),
    ..row(
        criterion: [Failure blast radius],
        relevance: [N/A],
        favors: [Decentralized],
        rationale: [A bug or crash in the central scheduler halts all
            reconciliation. A failure in one decentralized reconciler is
            isolated to that resource type. Not a factor because all reconcilers
            are required to work for the system to work too.],
    ),
)
- On a choisi l'approche centralisé. Cette approche ne scale pas du tout, mais c'est pas un problème, car le système n'est pas distribué et n'a pas pour but d'executer des milliers de conteneurs sur un seul système. Pour rappel on veut un minimum d'overhead. Si on déploie des milliers de conteneurs c'est qu'on a plein de resources et qu'on peut utiliser un orhcestrateur tel que K8S.
- Outre ça, le design la logique de réconciliation est fait de manière à être adaptable plus ou moins facilement a l'un ou l'autre des système.
- Le poitn d'entrée de la fonction

= Configuration du système

La gestion du système repose sur le principe de déclarativité: l'utilisateur
final décrit l'état dans lequel il souhaite que le système soit, et le système
va tetner de faire converger l'état désiré et l'état courrant. Ce principe est
largement employé dans les domaine adjacent à la conteneurisation comme au sein
de Kubernetes et Docker Compose. Dans le cadre d'un système d'exploitation,
plusieurs éléments sont a configurer, et ils seront découpés en "resources". Une
resource peut être une interfaces réseau, routes IP, conteneurs, volumes de
stockage, etc. Plus spécifiquement, une resource est quelque chose qui peut être
géré de manière indépendante [c'est pas clair].

Cette approche a deux avantages principaux: d'une part il est beaucoup plus
simple de comprendre le système et d'avoir une vue d'ensemble de sa
configuration. D'autre part, décrire le système sous forme de configuration
déclarative permet d'aisément versionner le tout dans Git ou autre système de
gestion. En effet, étant donné que le système s'occupe faire converger la
configuration, etc etc

= Réconciliation des resources

La configuration utilisateur est immutable, toutefois l peut être désirable de
joindre plusieurs resources "réeles" d'une seule resource exposée à
l'utilisateur pour les besoin de la configuration. Lorsqu'une telle resource est
présente, celle-ci donnera lieu donc a plusieurs sous-resources. La possibilité
au resources "dynamiques" de crée d'autre resources dynamiques enfant a aussi
été implémenté

Outre l'aspect de tracabilité et d'origin des resources, cette hiérarchie
parent/enfant permet aussi d'ordonnancer la supression des resources

== Orchestration de la réconciliation

Deux mécanismes peuvent initier une réconciliation: d'une part, une modification
de la configuration soumise par l'utilisateur via l'API laquelle doit être
propagée aux gestionnaires concernés; d'autre part, un événement propre à une
ressource, détecté directement par le gestionnaire qui en est responsable (crash
de conteneur, changement d'état réseau, etc.). Ces deux sources d'évenements
soulèvent une question architecturale fondamentale:

Ces deux sources d'événements soulèvent une question architecturale fondamentale
: qui orchestre la réconciliation ? Autrement dit, quel composant est
responsable d'initier la réconciliation d'une ressource donnée, et à quel moment
?






Une fois le systèem ayant convergé, il est possible que l'état dérive a nouveau.
Par exemple, lorsqu'un conteneur s'interrompt ou plante, il est nécessaire de le
remttre en route. Cela peut aussi être le cas avec les interfaces réseau ou le
*hot-plugigng* de disques. Bien que cela puisse se faire de manière entièrement
évenementielle, il a été décidé que, ppour des raisons de simplicité, une
réconciliation serait executée de manière récurrente


NOTE: en gros le problème c'est que la config arrive dnas un endroit centralisé
(l'API) qui doit notifier bah les gestionnaire, mais de l'autre coté les
gestionnaires ont aussi des évenemetns propre à leur resource. Du coup la
questino se pose de "Qui orchestre la réconciliation (i.e. qui initie la
réconciliation d'une resource donnée)"

= Réconciliation des resources

Chaque type de resource possède sa propre logique de réconciliation, toutefois,
totues suivent une procédure commune. La procédure est initialisé par la
commande de réconciliation, celle si transmet



= Réconciliation d'une resource

Chaque type de resource dispose de sa propre logique de réconciliation,
toutefois, elles suivent toutes une procédure commune. La commande qui initie la
réconciliation contient l'identité de la resource à réconcilier, sont état
désiré (spécification), ainsi que l'état actuel calculé durant l'appel
précédent.

Tout d'abord l'état actuel va être recalculé via `refresh()`. Ensuite, `plan()`
va comparer le nouvel état actuel avec l'état désiré et sortir un plan qui
représente les actions a entreprendre _immédiatement_ pour faire converger les
deux. Il est important de noter ici que _immédiatement_ veut dire que l'ensemble
des actions doit être réalisable en un temps court. Si l'action est de longue
durée (par exemple un téléchargement), alors celle-ci constitue la seul action
du plan. Une fois le plan calculé, celui-ci peut être appliqué. Lorsque l'action
entreprise prende longtemps, alors celle-ci est lancée en arrière-plan, de sorte
à ce que la fonction `apply()` retourne dans un délai assez bref. Cette
exécution en arrière plan doit être fait de sorte a ce que un appel ultérieur a
`reconcile()` ne génère pas de nouvelles tâches d'arrière-plan si cela n'est pas
nécessaire (pour ne pas télécahrger un fichier deux fois par exemple).

Enfin, la fonction `update()` va prendre l'ensemble de ces données, et produire
une réponse à transmettre à l'appelant. Cette fonction est utile car, dans le
cadre d'un apply, c'est ici que l'état actuel peut être recalculé une seconde
fois, toutefois, si le plan ne possède aucun changement, alors on peut
réutiliser l'état que nous avions déjà récupérer dans `refresh()`.

= Orchestration de la réconciliation

Chaque resource dispose de sa propre logique de réconciliation, gérée au sein
d'un "manager". Afin d'avoir un système résilient, il est nécessaire d'apeller
cette logique de réconciliation de manière régulière [pourquoi?]. Pour ce faire,
deux approches fondamentalement opposées existent: une approche centralisée et
une approche décentralisée.

Par approche centralisée, il faut comprendre qu'un seul composant est
responsable de parcourir la liste des resources et de transmettre des demandes
de réconciliation au sous-système responsable. À l'inverse, dans l'approche
décentralisée, le sous-sytème responsable de la resource va lui-même planifier
sa propre boucle.

= Summary

== Issue

To prevent configuration drift and ensure that the system is self-healing, a
reconciliation loop needs to be implemented. Each tick of the loop will
+ fetch the desired state;
+ fetch the current state;
+ act;
+ update the status.

The loop can either be implemented on the "state manager" (centralized), or
within each controller (decentralized).

== Decision

The reconciliation loop will be implemented in the state manager.

The state manager will own scheduling and orchestration, and will call
`reconcile(Resource)` for each resource. Each reconciler will return the updated
status together with any requested creations, modifications, or deletions.

= Details

== Assumptions

- The system will only managed a limited number of reconcilable resources
- If any of the controler faults, the whole system has to fault

== Constraints

- _none_

== Positions

=== Centralized Scheduling

The state manager owns the reconciliation loop. It calls `reconcile(Resource)`
on each reconciler and waits for a response containing the updated status and
any pending mutations (create / update / delete).

=== Decentralized Scheduling

Each reconciler owns its loop and implements it as it sees fit, independently
fetching desired and current state on every tick.

#pagebreak(weak: true)
#set page(flipped: true)

== Analysis

#show table.cell.where(x: 0): strong
#show table.cell.where(y: 0): strong

#table(
    columns: (12em, auto, auto, 1fr),
    row-gutter: (2.2pt, auto),
    table.header[Criterion][Relevance][Favors][Rationale],
    ..row(
        criterion: [Global scheduling control],
        relevance: [MEDIUM],
        favors: [Centralized],
        rationale: [A single loop can optimize the reconciliation by taking into
            account child and dependencies.],
    ),
    ..row(
        criterion: [Scheduling flexibility],
        relevance: [MEDIUM],
        favors: [Decentralized],
        rationale: [Each reconciler can tune its own tick interval, backoff, and
            concurrency without coordinating with a central scheduler.],
    ),
    ..row(
        criterion: [Reacting to internal state events],
        relevance: [MEDIUM],
        favors: [Centralized],
        rationale: [Event-driven reconciliation (trigger on state change rather
            than timer) is straightforward to add to a central loop;
            decentralized reconcilers would each need independent subscription
        ],
    ),
    ..row(
        criterion: [Reaction to external events],
        relevance: [MEDIUM],
        favors: [Decentralized],
        rationale: [Events originating outside the state manager (e.g. a
            container crash) can be handled directly by the owning reconciler.
            In a centralized model, all external signals must be funnelled
            through the state manager
        ],
    ),
    ..row(
        criterion: [Stuck reconciler detection],
        relevance: [LOW],
        favors: [Centralized],
        rationale: [Because the central scheduler expects a response from
            `reconcile()`, it can detect a stuck reconciler via timeout.
            Decentralized reconcilers fail silently. Minor concern given the
            small resource count.],
    ),
    ..row(
        criterion: [Automated sub-resource ownership],
        relevance: [LOW],
        favors: [Centralized],
        rationale: [Because the `reconcile()` call also returns the creation
            requests, it is trivial to associate a parent/child relationship.],
    ),
    ..row(
        criterion: [API call overhead],
        relevance: [N/A],
        favors: [Centralized],
        rationale: [`reconcile(Resource)` bundles all necessary state into one
            call vs. multiple fetches per tick in the decentralized model. Not a
            factor given no API rate-limit constraints.],
    ),
    ..row(
        criterion: [Failure blast radius],
        relevance: [N/A],
        favors: [Decentralized],
        rationale: [A bug or crash in the central scheduler halts all
            reconciliation. A failure in one decentralized reconciler is
            isolated to that resource type. Not a factor because all reconcilers
            are required to work for the system to work too.],
    ),
)

#pagebreak(weak: false)
#set page(flipped: false)



= Aaaaaa

La gestion du système de manière déclarative se fait au travers d'un fichier de
configuration. Celui-ci décrit l'ensemble des aspects du système qui doivent
être configurés. Cela inclu la configuration d'une interface réseau, de routes
IP, de conteneurs, ou bien encore de disques.

Afin de faciliter la gestion de ces aspects, ceux-ci sont séparés en
"resources". Pour chaque resource, sa réconciliation passe par 4 étapes:
- le `refresh`, ou on va récupérer l'état actuel de la resource
- le `plan`, ou

== Composants

La solution est découpée en plusieurs composants. Chaque composant est
responsable d'un aspect particulier de la gestion du système. Un certains nombre
de composant peuvent être regroupés sous "managers". Ces "managers" sont chargés
de réconcilier l'état désiré du système avec l'état actuel et suivents tous une
architecture semblable qui sera expliqué plus en détails au chapitre XYZ.

La solution est composée de X composants:
+ `supervisor`
+ `system-manager`
+ `network-manager`
+ `container-manager`
+ `storage-manager`










= Conceptuelle

== Utilisation de la solution

L'administrateur du système crée un fichier de configuration, dans le cas le
plus simple, celui-ci contient:
- la configuration réseau



== Composants logiciel

=== Superviseur

Le superviseur est le premier exécutable appelé par le noyeau Linux, il doit,
dans l'ordre:
+ monter l'ensemble des pseudo-FS (càd `/dev`, `/sys`, etc.)
+ monter la configuration (sous `/etc/containers/config`)

Le superviseur est le premier exécutable appelé par le noyeau Linux, il doit:
- monter la configuration

=== System Manager
=== Network Manager
=== Container Manager

== Gestion de l'état

Deux architecture possible:
+ un composant central qui stocke et schedule les réconciliation (pour donner a
    d'autre composants)
+ ou, le composant central ne stoque que la data

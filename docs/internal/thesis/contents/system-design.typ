#import "../lib.typ": *

// TODO: parler des dépendances cycliques

#set heading(numbering: "1.")

= Conception du système

== Architecture générale
#todo[Architecture générale][
    Le schéma C4 mais en mieux
]

== Démarrage du système
#todo[Démarrage du système][
    Schéma du processus de boot (init -> supervisor -> les contrôleurs)
]

== Orchestration de la réconciliation

Le système est composé de plusieurs resources, chacune gérée par une boucle de
logique de réconciliation indépendantes. Une réconciliation peut arriver de
manière périodique ou en réponse à un événements, tant interne, tel qu'une mise
a jour de la configuration, qu'externe, comme un conteneur qui s'arrête. À ce
titre, deux modèles d'orchestration sont possible: un modèle centralisé, qui
contient une boucle unique, et au sein de cette boucle, fait appel a une méthode
de réconciliation sur chaque resource qui représente une itération. L'autre
modèle est le modèle décentralisé, dans lequel chaque contrôleur contient sa
propre boucle, organisé de la manière qui lui convient (une boucle par resource,
une boucle par type, etc.).

Le modèle centralisé dispose d'une vue globale de toutes les ressources et leur
dépendance, ce qui permet de mieux coordonner la réconciliation à travers
plusieurs contrôleurs. Dans le cadre de ce modèle, l'orchestrateur pourrait
décider de ne pas réconcilier une ressource tant que ces dépendances ne sont pas
prêtes. À l'inverse, dans le modèle décentralisé, la ressource serait quand même
réconciliée quoi qu'il arrive. Toutefois le modèle décentralisé permet une plus
grande flexibilité en termes de planification: chaque contrôleur peut déterminer
lui-même l'intervalle d'exécution, etc. Cela peut toutefois être partiellement
"mitigé" en ajoutant des paramètres contrôlant le timing au sein des réponses
liées à la réconciliation.

En outre, le modèle centralisé permet d'aisément réagir aux changements d'état
et de spécification des ressources. En effet, étant donné qu'un élément central
coordonne le tout, il est trivial de propager les changements d'état ou
re-déclencher les réconciliations dépendantes. Dans un modèle décentralisé, un
mécanisme de notification entre les différents contrôleurs serait nécessaire et
risque de complexifier le système. À l'inverse, le modèle décentralisé permet de
réagir de manière beaucoup plus simple aux événements "externes" (par exemple,
un conteneur qui s'arrête ou une interface réseau qui tombe). Pour supporter de
genre de cas dans le modèle centralisé, il serait nécessaire d'ajouter une
procédure permettant de notifier l'orchestrateur central.

En outre l'approche centralisée propose deux autres avantages mineurs: d'une
part cela permet d'assigner trivialement le parent d'une ressource; en effet,
étant donné que l'appel de réconciliation est effectué sur une ressource connue,
et que sa réponse contient les sous-ressources a créé, il n'y a pas de risque
qu'un parent soit mal assigné. En outre, l'orchestrateur centralisé permet
d'aisément détecter lorsqu'un contrôleur est en panne: en effet, si celui-ci ne
répond pas dans le temps imparti, on peut considérer qu'il est bloqué et prendre
les actions appropriées.

Enfin, deux points supplémentaires existent, mais ne sont pas pris en compte:
d'une part le modèle centralisé prend moins d'appels API (juste un),
contrairement au système décentralisé qui a besoin d'au moins deux appels (l'un
pour récupérer la config, l'autre pour push l'état). Cela n'est pas pris en
compte, car il y a au final peu de ressources et les contrôleurs sont tous
locaux, donc ce n'est pas important. Le modèle décentralisé permet une plus
grande robustesse, car la panne est limitée à un nombre limité de ressources.
Cela est toutefois sans importance, car les contrôleurs sont primordiaux et leur
panne, mais en péril le système entier, donc tout s'arrête.

#let row-label(body) = {
    set par(justify: false)
    body
}

#let row(criterion: [], pertinence: "", faveur: [], justification: []) = {
    (
        row-label(criterion),
        row-label(pertinence),
        row-label(faveur),
        justification,
    )
}

#set page(flipped: true)
#figure(
    label: <scheduling>,
    caption: [Comparaison des modèles centralisés et décentralisés],
    // TODO: ajouter note: [],
    source: made-by-self,
    {
        show table.cell.where(x: 0).or(table.cell.where(y: 0)): set text(
            weight: "bold",
        )

        table(
            columns: (7em, auto, auto, 1fr),
            row-gutter: (2.2pt, auto),
            align: start,
            table.header[Critère][Pertinence][Faveur][Justification],
            ..row(
                criterion: [Contrôle global de la planification],
                pertinence: [Moyenne],
                faveur: [Centralisé],
                justification: [Une boucle unique peut optimiser la
                    réconciliation en tenant compte des dépendances et de la
                    hiérarchie des ressources.],
            ),
            ..row(
                criterion: [Flexibilité de planification],
                pertinence: [Moyenne],
                faveur: [Décentralisé],
                justification: [Chaque contrôleur peut régler son propre
                    intervalle d'exécution, son backoff et sa concurrence sans
                    coordination centrale.],
            ),
            ..row(
                criterion: [Réaction aux événements internes],
                pertinence: [Moyenne],
                faveur: [Centralisé],
                justification: [Déclencher une réconciliation sur changement
                    d'état est trivial dans une boucle centrale; dans un modèle
                    décentralisé, chaque contrôleur devrait gérer ses propres
                    souscriptions.],
            ),
            ..row(
                criterion: [Réaction aux événements externes],
                pertinence: [Moyenne],
                faveur: [Décentralisé],
                justification: [Les événements externes (ex. arrêt d'un
                    conteneur) peuvent être traités directement par le
                    contrôleur concerné. Dans le modèle centralisé, ces signaux
                    doivent transiter par l'orchestrateur.],
            ),
            ..row(
                criterion: [Détection d'un contrôleur bloqué],
                pertinence: [Faible],
                faveur: [Centralisé],
                justification: [L'orchestrateur central peut détecter un
                    contrôleur bloqué via un timeout sur l'appel de
                    réconciliation. Les contrôleurs décentralisés échouent
                    silencieusement.],
            ),
            ..row(
                criterion: [Assignation automatique du parent],
                pertinence: [Faible],
                faveur: [Centralisé],
                justification: [La réponse de `reconcile()` contient les
                    demandes de création, ce qui permet d'associer trivialement
                    la relation parent-enfant.],
            ),
            ..row(
                criterion: [Nombre d'appels API],
                pertinence: [Non retenu],
                faveur: [Centralisé],
                justification: [`reconcile()` regroupe l'état en un seul appel,
                    contre plusieurs requêtes par itération en mode
                    décentralisé. Non pertinent étant donné le faible nombre de
                    ressources et l'absence de contrainte de débit.],
            ),
            ..row(
                criterion: [Périmètre d'une panne],
                pertinence: [Non retenu],
                faveur: [Décentralisé],
                justification: [Une panne de l'orchestrateur central arrête
                    toutes les réconciliations. En mode décentralisé, la panne
                    est isolée au contrôleur concerné. Non pertinent car la
                    panne de n'importe quel contrôleur met en péril le système
                    entier.],
            ),
        )
    },
)
#set page(flipped: false)

Au regard du @scheduling, le modèle centralisé a été retenu comme modèle
d'orchestration. Les avantages en termes de coordination des dépendances, de
réaction aux événements internes et de détection de pannes l'emportent sur la
flexibilité de planification et la capacité de réaction aux éventement externes
offerte par le modèle décentralisé. À ce titre, l'orchestrateur inclura donc un
mécanisme permettant de aux contrôleur de remonter les événements externes en
vue de lancer une nouvelle réconciliation.

== Modèle de ressources

=== Structure d'une resource
Chaque resource est composée d'un identifiant unique, d'une spécification (=
l'état désiré), et d'un état (= l'état actuel). En outre, une resource peut un
parent, des enfants, ou des dépendances. L'ensemble de ces données est stockées
et permet de constituer une resource.

La mise à jour de la spécification est décrite dans le @sub-restype. En ce qui
concerne la mise à jour de l'état, celui-ci est géré par un contrôleur qui
implémente la logique de réconciliation.

=== Types de resources <sub-restype>
Les resources réconciliables du système sont séparées en trois types, ce qui
permet de distinguer qui peut crée, modifier, ou supprimer une resource. Dans le
cadre de la création, le propriétaire est celui qui va ordonner la création de
la resource et qui en devient donc naturellement sont propriétaire. Un
propriétaire peut être interne au système, en particulier, il s'agit d'une autre
resource, ou externe au système, en particulier, l'administrateur du système,
via l'API. La @restype illustre les différents types de ressources ainsi que les
règles régissant leur création, leur modification et leur suppression selon la
nature de leur propriétaire.

#figure(
    label: <restype>,
    caption: [Types de resources et leur propriétés],
    note: [
        Les différents types de resources avec qui peut les créer ou les
        modifier, qui peut les supprimer, et ou se situe le propriétaire dans le
        système.
    ],
    source: made-by-self,
    {
        show table.cell.where(x: 0).or(table.cell.where(y: 0)): set text(
            weight: "bold",
        )

        table(
            columns: (auto, auto, 1fr, auto),
            rows: 1.5em,
            align: center + horizon,
            table.header(
                [Type],
                [Création et modification],
                [Suppression],
                [Nature du propriétaire],
            ),
            ..(
                [Statique],
                table.cell(colspan: 2, rowspan: 2)[Propriétaire],
                [Externe au système],
            ),
            [Dynamique],
            table.cell(rowspan: 2)[Interne au système],
            ..([Mutualisée], [Les propriétaires], [L'orchestrateur]),
        )
    },
)

=== Dépendances et hiérarchie
Une resource peut dépendre d'autres resources. Ce lien de dépendance implique
trois choses: d'une part, même si elle n'est pas propriétaire de la resource
peut acceder à l'état à la spécification de sa dépendance, sans pouvoir la
modifier. D'autre part, il sera impossible de supprimer complètement les
resources sur laquelle dépend la resource tant que cette dépendance existe.
Enfin, la resource sur laquelle il y a dépendance pourra acceder à la
spécification et a l'état de ces resources, toujours sans pouvoir les modifier.

Le liens de parenté est une version plus forte du lien de dépendance; un parent
est celui ayant crée la resource et peut donc la modifier et la supprimer. Ses
enfants sont aussi considérés comes des dépendances et donc bloquent la
suppression complète jusqu'à ce qu'ils aient été supprimés.

Enfin il reste le cas particulier des resources mutualisés; celles-ci possèdent
plusieurs parent, mais ceux-ci ne disposent que d'un pouvoir de création et de
modification, et non le pouvoir de suppression. À l'inverse, une resource
mutualisée ne peut pas être supprimées tant que ses parents en ont besoins.
Compte tenu de cela, les resources mutualisés sont aussi considérée comme des
racines, sur lesquelles dependents les "parents".

=== Suppression des resources
Lorsqu'une demande de suppression d'une ressource est enregistrée, le système va
marquer la resource cible comment devant être supprimée. Tous les enfants de la
resource, qui sont aussi des dépendances, sont récursivement marqués pour
suppression. Tant qu'une resource possède encore des dépendances, celle-ci ne
peut pas être complètement supprimées.

=== Resources mutualisée
Les resources mutualisé sont des resources qui sont partagées (donc sur
lesquelles d'aures resources dependents), et qui sont crée de manière implicite
par ces resources dépendantes. Dans la majorité des cas, une resource implicite
peut être supprimée au profit d'une resource statique ou dynamique sans péjorer
la facilité d'utilisation. Il existe certains cas ou cela n'est toutefois pas
possible, notament le cas pour les images de conteneurs. Bien que
"physiquement", le conteneur et l'image soit deux resources séparées mais liées,
la majorité des systèmes de conteneurisation permettent de gérer l'image
directement avec le conteneurs et opèrent en arrière plan pour dé-dupliquer le
tout. Ils offrent aussi la possibilité de gérer manuellement les images. En
contrepartie, il est nécessaire de disposer d'un mécanisme de "garbage
collection" qui supprimera les images non-utilisée, même lorsque celles-ci ont
été sciement télécharger. Il faut donc introduire le concept de resource
mutualisé.

Afin d'illustrer la problématique de ces resources, l'exemple des images de
conteneurs.

#set page(flipped: true)

#refdiagram(
    label: <cfgshared>,
    caption: [Problème de resources partagées],
    note: [
        Deux configuration utilisateur indépendantes, agissent au final sur une
        resource partagée au niveau du système en raison du fonctionnement de la
        resource réel.

        Note: la configuration est abrégée a des fins d'illustration
    ],
    source: made-by-self,

    spacing: 1cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgshared-cfga>, (0.875, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-a
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-imga>, (0.875, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Image
            name: container-a-img
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-runa>, (0, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-a-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-cfgb>, (1.85, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-b
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-imgb>, (1.85, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Image
            name: container-b-img
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-runb>, (2.825, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-b-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgshared-podimg>, (1.5, 3), stroke: 2pt, title: [
            /var/lib/container/image/alpine/latest
        ])

        node(label: <cfgshared-podruna>, (0, 3), stroke: 2pt, title: [
            Running Container A
        ])

        node(label: <cfgshared-podrunb>, (2.875, 3), stroke: 2pt, title: [
            Running Container A
        ])

        node(
            label: <cfgshared-cfg>,
            num: [1],
            enclose: (
                <cfgshared-cfga>,
                <cfgshared-cfgb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: blue,
            title: align(top + left, place(dx: 5cm, dy: 2.5cm, text(
                fill: blue,
            )[
                *User Configurations*
            ])),
        )

        node(
            label: <cfgshared-dyn>,
            num: [2],
            enclose: (
                <cfgshared-imga>,
                <cfgshared-runa>,
                <cfgshared-imgb>,
                <cfgshared-runb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(
                fill: red,
            )[
                *Dynamic Resources*
            ])),
        )

        node(
            label: <cfgshared-real>,
            num: [3],
            enclose: (
                <cfgshared-podruna>,
                <cfgshared-podrunb>,
                <cfgshared-podimg>,
            ),
            inset: 2mm,
            snap: false,
            stroke: orange,
        )

        node(
            label: <cfgshared-reallabel>,
            (rel: (0mm, -1cm), to: <cfgshared-real>),
            title: text(fill: orange)[*Physical Resources*],
        )

        edge(<cfgshared-cfga>, <cfgshared-imga>, "-|>")
        edge(<cfgshared-cfga>, <cfgshared-runa>, "-|>")
        edge(<cfgshared-runa>, <cfgshared-podruna>, "-|>")
        edge(
            <cfgshared-imga>,
            <cfgshared-podimg>,
            "-|>",
            label: <cfgshared-conflict>,
            num: [4],
        )

        edge(<cfgshared-cfgb>, <cfgshared-imgb>, "-|>")
        edge(<cfgshared-cfgb>, <cfgshared-runb>, "-|>")
        edge(<cfgshared-runb>, <cfgshared-podrunb>, "-|>")
        edge(<cfgshared-imgb>, <cfgshared-podimg>, "-|>")

        edge(<cfgshared-podruna>, <cfgshared-podimg>, "--|>")
        edge(<cfgshared-podrunb>, <cfgshared-podimg>, "--|>")
    },
)

La @cfgshared montre que, dans le cas où l'utilisateur souhaite configurer deux
conteneurs reposant sur l'image~`alpine:latest`~#bref(<cfgshared-cfg>). Chaque
configuration génère un ensemble de sous-ressources disposant chacune de sa
propre boucle de réconciliation~#bref(<cfgshared-dyn>) gérant une resource
physique~#bref(
    <cfgshared-real>,
). Or, certaines de ces sous-ressources, bien qu'indépendantes du point de vue
logique, gèrent en réalité la même ressource physique sous-jacente~#bref(
    <cfgshared-conflict>,
). Il en résulte un conflit potentiel car la resourcere serait géré par deux
boucles différentes n'ayant pas conaissance l'une de l'autre.

#set page(flipped: false)

Si l'utilisateur supprime l'une des deux configurations, la boucle de
réconciliation associée tentera de détruire la ressource partagée, alors que
celle-ci est encore requise par l'autre configuration. Selon la nature de la
ressource sous-jacente, deux comportements sont possibles: soit la suppression
échoue et la réconciliation reste bloquée, soit la ressource est détruite et
doit être recréée par l'autre configuration.

La solution est donc d'introduire la ressource mutualisée du @sub-restype: ce
type de ressource dispose de plusieurs parents pouvant la créer, mais son
identifiant constitue à lui seul sa spécification complète. Tout désaccord entre
parents se traduit donc par la création de ressources distinctes plutôt que par
un conflit. Dans l'exemple des conteneurs, la réconciliation se ferait donc
comme suit:
+ La resource statique correspondant à la configuration utilisateur va crée une
    sous-resource dynamique privée de type `ImageRef` en lui passant l'image,
    puis suspend son exécution jusqu'à ce que la resource `ImageRef` ait un
    état.

+ La resource `ImageRef` va faire une requête sur le registre de conteneur afin
    de déterminer l'identifiant unique. Une fois cela fait, cet identifiant est
    enregistré dans l'état de la resource. De plus, elle va déclarer une
    resource partagée de type `Image` faisant directement référence a l'image
    ainsi que l'identifiant unique et en fait donc une dépendance.

+ La resource `Image` est réconciliée normalement

+ Lors de la prochaine réconciliation de `Container`, le contrôleur pourra
    accéder à l'identifiant unique de l'image dans `ImageRef` et crée un
    conteneur se basant sur celui-ci.

+ Lors de la suppression du `Container`, tout d'abord le conteneur a proprement
    parlé sera supprimé, une fois cela fait, `ImageRef` est supprimé. Dès lors,
    l'orchestrateur verra que plus rien ne dépend sur `Image` et planifiera donc
    sa suppression.

+ Dans le cas ou d'autres conteneurs dépenderaient sur `Image`, alors rien de
    plus ne se passerait.

Comme illustré dans la @cfgjoint.

#set page(flipped: true)

#refdiagram(
    label: <cfgjoint>,
    caption: [Solution aux resources partagées],
    note: [
        Deux configuration utilisateur indépendantes, agissent au final sur une
        resource partagée au niveau du système en raison du fonctionnement de la
        resource réel.

        Note: la configuration est abrégée a des fins d'illustration
    ],
    source: made-by-self,

    spacing: 1cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgjoint-cfga>, (0.875, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-a
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-imga>, (0.875, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ImageRef
            name: container-a-img
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-runa>, (0, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-a-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-cfgb>, (1.85, 0), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: Container
            name: container-b
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-imgb>, (1.85, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ImageRef
            name: container-b-img
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-runb>, (2.825, 2), title: box(
            width: 6cm,
        )[
            ```yaml
            kind: ContainerRun
            name: container-b-run
            image: alpine:latest
            ```
        ])

        node(label: <cfgjoint-img>, (1.5, 3), title: box(
            width: 8cm,
        )[
            ```yaml
            kind: Image
            name: alpine:latest@sha256:AAAAA
            ```
        ])

        node(label: <cfgjoint-podimg>, (1.5, 4), stroke: 2pt, title: [
            /var/lib/container/image/alpine/latest
        ])

        node(label: <cfgjoint-podruna>, (0, 4), stroke: 2pt, title: [
            Running Container A
        ])

        node(label: <cfgjoint-podrunb>, (2.875, 4), stroke: 2pt, title: [
            Running Container A
        ])

        node(
            label: <cfgjoint-cfg>,
            enclose: (
                <cfgjoint-cfga>,
                <cfgjoint-cfgb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: blue,
            title: align(top + left, place(dx: 5cm, dy: 2.5cm, text(
                fill: blue,
            )[
                *User Configurations*
            ])),
        )

        node(
            label: <cfgjoint-dyn>,
            enclose: (
                <cfgjoint-imga>,
                <cfgjoint-runa>,
                <cfgjoint-imgb>,
                <cfgjoint-runb>,
            ),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(
                fill: red,
            )[
                *Dynamic Resources*
            ])),
        )

        node(
            label: <cfgjoint-real>,
            enclose: (
                <cfgjoint-podruna>,
                <cfgjoint-podrunb>,
                <cfgjoint-podimg>,
            ),
            inset: 2mm,
            snap: false,
            stroke: orange,
        )

        node(
            label: <cfgjoint-reallabel>,
            (rel: (0mm, -1cm), to: <cfgjoint-real>),
            title: text(fill: orange)[*Physical Resources*],
        )

        node(
            label: <cfgjoint-joint>,
            num: [2],
            enclose: (
                <cfgjoint-img>,
            ),
            inset: 2mm,
            snap: false,
            stroke: fuchsia,
            title: align(top + left, place(dx: -3.5cm, dy: 0cm, text(
                fill: fuchsia,
            )[
                *Mutual Resources*
            ])),
        )

        edge(<cfgjoint-cfga>, <cfgjoint-imga>, "-|>")
        edge(<cfgjoint-cfga>, <cfgjoint-runa>, "-|>")
        edge(<cfgjoint-runa>, <cfgjoint-podruna>, "-|>")
        edge(
            <cfgjoint-imga>,
            <cfgjoint-img>,
            "-|>",
            num: [1],
            label: <cfgjoint-imgref>,
        )

        edge(<cfgjoint-cfgb>, <cfgjoint-imgb>, "-|>")
        edge(<cfgjoint-cfgb>, <cfgjoint-runb>, "-|>")
        edge(<cfgjoint-runb>, <cfgjoint-podrunb>, "-|>")
        edge(<cfgjoint-imgb>, <cfgjoint-img>, "-|>")

        edge(
            <cfgjoint-img>,
            <cfgjoint-podimg>,
            "-|>",
            num: [3],
            label: <cfgjoint-noconflict>,
        )

        edge(<cfgjoint-podruna>, <cfgjoint-podimg>, "--|>")
        edge(<cfgjoint-podrunb>, <cfgjoint-podimg>, "--|>")
    },
)

Dans la @cfgjoint, les deux conteneurs vont, à travers une `ImageRef`~#bref(
    <cfgjoint-imgref>,
), finalement créer et gérer une même resource logique~#bref(<cfgjoint-joint>),
qui sera seule responsable de la resource physique~#bref(<cfgjoint-noconflict>).
Le problème apparaît ici comme étant déplacé ailleur plutôt que résolu. La
résolution concrète vient du fait que la resource mutualisé~#bref(
    <cfgjoint-joint>,
) ne possède pas de paramètres, et il ne peut donc pas y avoir de conflit; si
une resource change le nom, cela va simplement crée une nouvelle resource.

#set page(flipped: false)

=== Synthèse du model de resource

// En somme, deux liens existent entre les resources: le lien de parenté, qui
// permet de modifier et lire une resource, et le lien de dépendance, qui permet de
// lire une resource sans pouvoir y modifier. En outre, le lien de dépendance a un
// effet bloquant, empêchant la resource cible d'être supprimée tant qu'une autre
// resource dépend sur elle.

// En somme, les resources sont décomposée en 3 catégories: statiques, dynamiques,
// et mutualisées. Deux liens existent entre les resources: le lien de parenté, et le lien de dépendance: le

Ces catégories régissent qui peut executer quelles actions sur ces dernières, en
particulier une resource statique ne peut être crée et modifiée que par
l'administrateur système, une resource dynamique peut être crée par n'importe
quelle autre resource et modifiée uniquement par celle qui l'a crée, et enfin

#let static(..args) = node(stroke: blue, ..args)
#let dyn(..args) = node(stroke: red, ..args)
#let shared(..args) = node(stroke: fuchsia, ..args)
#let rel-rwd(parent, child, ..args) = edge(
    parent,
    child,
    "-|>",
    stroke: red,
    ..args,
)
#let rel-r(from, to, ..args) = edge(from, to, "--|>", stroke: green, ..args)
#let rel-rw(from, to, ..args) = edge(from, to, "-x-|>", stroke: fuchsia, ..args)

#refdiagram(
    label: <cfgtree>,
    caption: [Dérivation de ressources dynamiques depuis une configuration
        réseau],
    note: [
        À partir d'une unique configuration réseau, le contrôleur dérive
        automatiquement trois ressources dynamiques correspondant aux objets
        qu'il manipule au sein du noyau Linux.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 1pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        static(label: <cfgtree-dns>, (0, -1), title: [dns])
        dyn(label: <cfgtree-dnsfile>, (0, -2), title: [file:/etc/resolv.conf])

        static(
            label: <cfgtree-neta>,
            (0, 0),
            title: [interface:eth0],
        )
        dyn(
            label: <cfgtree-neta-addr>,
            (rel: (0, 1), to: <cfgtree-neta>),
            title: [address],
        )
        dyn(
            label: <cfgtree-neta-route>,
            (rel: (-0.75, 1), to: <cfgtree-neta>),
            title: [route],
        )
        dyn(
            label: <cfgtree-neta-link>,
            (rel: (0.75, 1), to: <cfgtree-neta>),
            title: [link],
        )

        static(
            label: <cfgtree-cona>,
            (2, 0),
            title: [container:test-a],
        )
        dyn(
            label: <cfgtree-cona-run>,
            (rel: (-0.5, 1), to: <cfgtree-cona>),
            title: [container-instance],
        )
        dyn(
            label: <cfgtree-cona-img>,
            (rel: (0.75, 1), to: <cfgtree-cona>),
            title: [image-ref],
        )

        static(
            label: <cfgtree-conb>,
            (2, -1),
            title: [container:test-b],
        )
        dyn(
            label: <cfgtree-conb-run>,
            (rel: (-0.5, -1), to: <cfgtree-conb>),
            title: [container-instance],
        )
        dyn(
            label: <cfgtree-conb-img>,
            (rel: (0.75, -1), to: <cfgtree-conb>),
            title: [image-ref],
        )

        shared(
            label: <cfgtree-img>,
            (3, -0.5),
            title: [image],
        )

        rel-rwd(<cfgtree-dns>, <cfgtree-dnsfile>)

        rel-rwd(<cfgtree-neta>, <cfgtree-neta-link>)
        rel-rwd(<cfgtree-neta>, <cfgtree-neta-addr>)
        rel-rwd(<cfgtree-neta>, <cfgtree-neta-route>)

        rel-r(<cfgtree-neta-addr>, <cfgtree-neta-link>)
        rel-r(<cfgtree-neta-route>, <cfgtree-neta-addr>)

        rel-rwd(<cfgtree-cona>, <cfgtree-cona-run>)
        rel-rwd(<cfgtree-cona>, <cfgtree-cona-img>)
        rel-r(<cfgtree-cona-run>, <cfgtree-cona-img>)

        rel-rwd(<cfgtree-conb>, <cfgtree-conb-run>)
        rel-rwd(<cfgtree-conb>, <cfgtree-conb-img>)
        rel-r(<cfgtree-conb-run>, <cfgtree-conb-img>)

        rel-rw(<cfgtree-cona-img>, <cfgtree-img>)
        rel-rw(<cfgtree-conb-img>, <cfgtree-img>)

        rel-rwd(
            (3, 2.5),
            (4, 2.5),
            title: [Crée, modifier, lire, supprimer],
            floating: true,
        )
        rel-r((3, 3), (4, 3), title: [Lire], floating: true)
        rel-rw((3, 3.5), (4, 3.5), title: [Crée, lire])

        static((2, 2.5), title: [Resource statique])
        dyn((2, 3), title: [Resource dynamique])
        shared((2, 3.5), title: [Resource mutualisée])
    },
)

== Installation du système

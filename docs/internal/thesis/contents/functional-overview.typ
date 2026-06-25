#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

= Présentation du système
// Contient tout ce qui est important pour l'utilisateur final.

// Objectifs du chapitre
// Plan du chapitre
// Résultats du chapitre

== Principes

#show heading.where(level: 3): set heading(outlined: false)

// TODO: référencer le projet de semestre
Le système s'articule autour de cinq principes fondamentaux: la déclarativité,
l'automatisation, la légèreté, la sécurité par défaut et la portabilité. Ces
principes ont été dégagés de l'analyse des besoins présentés dans le projet de
semestre, et répondent à un contexte de déploiement particulier: des
environnements hétérogènes, non distribués, parfois sur du matériel aux
ressources contraintes et administrés par un utilisateur seul.

=== Déclarativité

Tout d'abord, plutôt que de décrire les étapes à exécuter pour atteindre un état
donné, l'administrateur décrit directement l'état final souhaité. Le système
détermine lui-même les actions nécessaires pour y parvenir, quelle que soit
l'état courant de la machine.

// TODO: Besoin de dire pourquoi ça facilite le retour arrière?
Ce choix est particulièrement adapté à un contexte de reconfiguration fréquente
sur des environnements variés. Une approche impérative obligerait
l'administrateur à prendre en compte l'état courant avant chaque opération et
rend plus difficile la récupération en cas d'erreur durant la configuration du
système. La déclarativité permet d'améliorer la reproductibilité des
déploiements et simplifie le retour à un état antérieur en cas d'erreur humaine.

=== Automatisation

Toute opération administrative, depuis l'installation initiale jusqu'à la
reconfiguration, doit pouvoir être réalisée sans intervention manuelle.
L'objectif recherché est de permettre une intégration simple dans le processus
de déploiement, en particulier lorsque celui-ci repose sur une approche
GitOps/DevOps.

Les interventions manuelles pour le diagnostic ou la récupération restent
possibles, mais doivent demeurer exceptionnelles et clairement distinctes du
mode normal d'exploitation.

=== Légèreté

Le système doit présenter une empreinte mémoire minimale afin de laisser un
maximum de ressources aux services déployés par l'administrateur. De fait, il
n'assume qu'un seul rôle: servir de plateforme d'exécution pour des services
conteneurisés et aucun composant superflu n'est présent.

Ce principe se justifie par des déploiement sur des appareils basse consommation
disposant de peu de resources. Cela présente en outre deux effets secondaires
bénéfiques: une configuration plus simple en raison du petit nombre de
composants et une surface d'attaque plus réduite.

=== Sécurité par défaut

Le système adopte des paramètres restrictifs dès l'installation, sans
configuration supplémentaire. Les services internes et ceux déployés par
l'administrateur sont isolés aussi fortement que possible, afin de limiter les
risques de mouvement latéral en cas de compromission d'un composant.

L'analyse des besoins révèle principalement un usage pour l’hébergement de
services accessible via Internet et donc soumis en permanence à des tentatives
d'intrusion. Des mécanismes d'exception strictement contrôlés restent
disponibles pour les cas qui le requièrent, notamment lorsque l'hôte assume un
rôle de routeur ou de pare-feu réseau.

=== Portabilité

L'interface d'administration et le format de configuration doivent rester
identiques quelle que soit la configuration matérielle ou réseau sous-jacente.
En dehors des dépendances introduites explicitement par l'administrateur, le
transfert du système d'un environnement à un autre ne doit nécessiter aucune
adaptation de la configuration existante.

Dans un contexte où un même administrateur administre plusieurs machines aux
profils distincts, cette homogénéité réduit la charge cognitive et rend les
procédures opérationnelles transférables sans friction.

#show heading.where(level: 3): set heading(outlined: true)

== Gestion déclarative

// TODO: référencer Terraform, Docker, et K8s
La gestion et la configuration du système reposent sur le principe de
déclarativité: l'administrateur décrit les divers éléments de configuration du
système au sein d'un fichier de configuration, téléverse ce fichier sur le
système, et celui-ci tente alors automatiquement et continuellement de faire
converger la configuration fournie avec l'état réel du système; c'est la
réconciliation. Cette approche est déjà bien connue dans le contexte de la
conteneurisation et de l'infrastructure, notamment au travers d'outils tels que
Terraform, Kubernetes ou Docker Compose qui adoptent aussi le concept de
déclarativité.

// TODO: référencer GitOps et Git (?)
Cette approche présente deux avantages principaux. D'une part, elle permet de
facilement comprendre le système et d'avoir une vue d'ensemble de sa
configuration à tout instant. D'autre part, elle se prête naturellement à une
gestion de type GitOps: le fichier de configuration constituant la source de
vérité unique du système, il suffit de versionner ce fichier dans Git pour
disposer d'un historique complet de l'infrastructure. Appliquer un changement
revient alors simplement à soumettre une nouvelle version du fichier, sans avoir
à se soucier de l'état précédent du système, puisque c'est le système lui-même
qui se charge de calculer et d'exécuter les actions nécessaires pour atteindre
le nouvel état désiré.

// TODO: référencer la théorie du contrôle
Ce principe est directement inspiré de la théorie du contrôle, et plus
précisément des boucles de rétroaction en circuit fermé. Au sein du présent
système, la configuration est décomposée en resources. Celles-ci représente une
interface réseau, une route IP, un disque, un conteneur, etc. Chaque resource
est associée à un contrôleur qui implémente la logique nécessaire afin de la
réconcilier. Le processus typique de réconciliation est illustrée par la boucle
de contrôle présentée dans la @ctrlloop.

#refdiagram(
    label: <ctrlloop>,
    caption: [Schéma conceptuel d'une boucle de contrôle déclarative],
    note: [
        Le contrôleur (encadré rouge) observe l'état actuel de la ressource,
        calcule l'écart avec l'état désiré, et applique les actions correctives.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <ctrlloop-obs>, num: [1], (0, 2), title: [Observe])
        node(label: <ctrlloop-diff>, num: [2], (1, 1), title: [Diff & Plan])
        node(label: <ctrlloop-act>, (2, 2), title: [Act])
        node(
            enclose: (<ctrlloop-obs>, <ctrlloop-diff>, <ctrlloop-act>),
            inset: 5mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Declarative Control Loop*
            ])),
        )
        node(
            label: <ctrlloop-cfg>,
            num: [3],
            stroke: none,
            (1, 0),
            title: [Desired State],
            subtitle: [user configuration],
        )
        node(
            label: <ctrlloop-res>,
            (1, 3),
            title: [Managed Resource],
            subtitle: [actual state],
        )

        edge(<ctrlloop-cfg>, <ctrlloop-diff>, "--|>")
        edge(
            <ctrlloop-obs>,
            <ctrlloop-diff>,
            "-|>",
            bend: 30deg,
            title: [Current state],
        )
        edge(
            label: <ctrlloop-actions>,
            num: [4],
            <ctrlloop-diff>,
            <ctrlloop-act>,
            "-|>",
            bend: 30deg,
            title: place(dx: 0.3em, box(
                fill: white,
                width: 5cm,
                outset: 2mm,
                place(dy: -0.45em)[Actions to close the gap],
            )),
        )
        edge(<ctrlloop-act>, <ctrlloop-obs>, "-|>", title: [Infinitely
            recurring])
        edge(<ctrlloop-obs>, <ctrlloop-res>, "--|>", label-side: right, title: [
            Gather information
        ])
        edge(<ctrlloop-act>, <ctrlloop-res>, "--|>", label-side: left, title: [
            Execute actions
        ])
    },
)

Par exemple, un utilisateur peut vouloir que son interface réseau soit "up", ce
qui constitue l'état souhaité~#bref(<ctrlloop-cfg>). L'état actuel de
l'interface (statut, adresse IP, etc.) est d'abord récupéré~#bref(
    <ctrlloop-obs>,
), puis comparé au statut désiré~#bref(<ctrlloop-diff>). Dans le cas où
l'interface se trouve dans l'état "down", la commande correspondante est
exécutée afin de la mettre en route~#bref(<ctrlloop-actions>). Ce même mécanisme
s'applique à toute mise à jour de la ressource: une modification de la
configuration déclarative se traduit automatiquement par les actions correctives
adéquates. Le processus se répétant indéfiniment, le système détecte et corrige
sans intervention tout écart causé par un facteur externe, tel que le
débranchement accidentel du câble.

// == Resources et contrôleurs <functional-overview-resource-model>
// #todo[][
//     - Présenter un peu plus les resources (pas forcément exhaustif)
//     - Présenter le split user vs dynamic
//     - Le concept de manager
// ]

Chaque resource est uniquement identifiable par la combinaison de son type et de
son nom. À chaque type de resource est associé un schéma décrivant les éléments
de configuration propre à la resource concernée, lorsque l'on fournit des
valeurs, cela devient alors une spécification et constitue l'état désiré. Outre
cette spécification, il est aussi possible de lire l'état actuel de la resource
qui est mis a jour de manière régulière par le système. Plus particulièrement,
cette mise à jour est effectuée par un contrôleur qui, comme expliqué dans le
/* TODO */, est responsable de la logique de réconciliation. C'est durant la
réconciliation que le contrôleur mettre à jour l'état actuel.

== Configuration
La configuration du système est simplement une liste de l'ensemble des resources
que l'utilisateur a explicitement configuré. Lorsqu'une resource disparait de
cette configuration, elle est alors supprimée. Il est important ici de noter
qu'une resource au sein du système peut être une abstraction ou un agrégat de
plusieurs resources physique. Ce mécanisme est illustré dans la @cfgdyn: une
entité de configuration unique peut être décomposée par le système en plusieurs
ressources dynamiques distinctes.

#refdiagram(
    label: <cfgdyn>,
    caption: [Dérivation de ressources dynamiques depuis une configuration
        réseau],
    note: [
        À partir d'une unique configuration réseau, le contrôleur dérive
        automatiquement trois ressources dynamiques correspondant aux objets
        qu'il manipule au sein du noyau Linux.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <cfgdyn-cfg>, num: [1], (0, 0), title: box(width: 6cm)[
            #codly(
                header: [User Configuration],
                highlighted-lines: (
                    (2, aqua.lighten(60%)),
                    (3, green.lighten(60%)),
                    (4, yellow.lighten(60%)),
                ),
            )
            ```yaml
            kind: Network
            name: eth0
            up: true
            address: 10.194.1.42/24
            ```
        ])

        node(label: <cfgdyn-link>, (1, 1), title: box(
            width: 6cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, aqua.lighten(60%)),
                    (4, green.lighten(60%)),
                ),
            )
            ```yaml
            kind: Link
            name: dyn-eth0-link
            match: eth0
            up: true
            ```
        ])

        node(label: <cfgdyn-addr>, (0.875, 2), title: box(
            width: 6cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, aqua.lighten(60%)),
                    (4, yellow.lighten(60%)),
                ),
            )
            ```yaml
            kind: Address
            name: dyn-eth0-addr
            link: eth0
            address: 10.194.1.42/24
            ```
        ])

        node(label: <cfgdyn-rte>, (0, 2), title: box(
            width: 6cm,
        )[
            #codly(
                highlighted-lines: (
                    (3, yellow.lighten(60%)),
                    (4, yellow.lighten(60%)),
                ),
            )
            ```yaml
            kind: Route
            name: dyn-eth0-rte
            network: 0.0.0.0/0
            via: 10.194.1.1
            ```
        ])

        node(
            label: <cfgdyn-dyn>,
            num: [2],
            enclose: (<cfgdyn-link>, <cfgdyn-addr>, <cfgdyn-rte>),
            inset: 2mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Dynamic Resources*
            ])),
        )

        edge(<cfgdyn-cfg>, <cfgdyn-link>, "-|>")
        edge(<cfgdyn-cfg>, <cfgdyn-addr>, "-|>")
        edge(<cfgdyn-cfg>, <cfgdyn-rte>, "-|>")
    },
)

Par exemple, dans le cas d'une configuration réseau, il est plus naturel de
penser la configuration comme étant l'interface, l'address, et le routage comme
une seule entité~#bref(<cfgdyn-cfg>), bien que cela soit en réalité trois entité
distinctes~#bref(<cfgdyn-dyn>). Dès lors, le concept de resources dynamiques est
introduit: là ou la configuration utilisateur contient des resources statiques,
ne pouvant être modifié que par l'administrateur, les resources dynamiques sont
crée à la volée par le système en réponse à la configuration utilisateur. Le but
est de permettre l'isolation stricte de la logique de réconciliation, tout en
permettant un usage simplifié du système.

Il est important de noter que, dans le cas des resources dynamiques,
l'administrateur ne peut pas les modifier directement; toute modification doit
passer à travers la modification de la resource statique mère. Il est toutefois
possible d'accèder à l'état de ces resources dynamiques.

== Interface d'administration <functional-overview-admin-api>

Étant donné le model de configuration, il n'est plus nécessaire d'executer des
commandes sur la machine. Il suffit d'uploader un fichier. Compte tenu de cela,
l'accès SSH est remplacé par une API classique, elle-même exposée via un client
en ligne de commande ainsi qu'un provider Terraform.

Outre le téléversement de fichiers, cette API permet aussi d'executer quelques
actions impératives (redémarrer un conteneur), récupérer et observer l'état
d'une resource (qu'elle soit dynamique ou non), voir les logs, et arrêter la
machine. Elle permet aussi de rentrer dans l'environnement d'exécution d'un
conteneur (equivalent de `docker exec`), de port forward, etc. Elle permet aussi
d'explorer le système de fichier et d'éditer certains fichier uniquement si
ceux-ci ne sont aps géré par une configuration (par exemple modifier un fichier
au sein d'un volume de conteneur). Prend en charge aussi les backups.

Dans le cadre du mode de maintenance, d'autre commandes pour le diagnostic sont
mise a disposition a travers un shell simple. Cela nécessite toutefois un
redémarrage.

== Exemple d'utilisation <functional-overview-example>
#todo[][
    Un exemple simple et concret pour bien cerner
]

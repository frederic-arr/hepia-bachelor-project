// #import "@preview/codly:1.3.0": *
// #import "@preview/codly-languages:0.1.1": *

// #set text(lang: "fr", hyphenate: false)
// #set par(justify: true)
// #set page(numbering: "1")

#import "../lib.typ": *

// #show: codly-init.with()
// #codly(languages: codly-languages)

= Conception du système
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
donné, l'utilisateur décrit directement l'état final souhaité. Le système
détermine lui-même les actions nécessaires pour y parvenir, quelle que soit
l'état courant de la machine.

// TODO: Besoin de dire pourquoi ça facilite le retour arrière?
Ce choix est particulièrement adapté à un contexte de reconfiguration fréquente
sur des environnements variés. Une approche impérative obligerait l'utilisateur
à prendre en compte l'état courant avant chaque opération et rend plus difficile
la récupération en cas d'erreur durant la configuration du système. La
déclarativité permet d'améliorer la reproductibilité des déploiements et
simplifie le retour à un état antérieur en cas d'erreur humaine.

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
maximum de ressources aux services déployés par l'utilisateur. De fait, il
n'assume qu'un seul rôle: servir de plateforme d'exécution pour des services
conteneurisés et aucun composant superflu n'est présent.

Ce principe se justifie par des déploiement sur des appareils basse consommation
disposant de peu de resources. Cela présente en outre deux effets secondaires
bénéfiques: une configuration plus simple en raison du petit nombre de
composants et une surface d'attaque plus réduite.

=== Sécurité par défaut

Le système adopte des paramètres restrictifs dès l'installation, sans
configuration supplémentaire. Les services internes et ceux déployés par
l'utilisateur sont isolés aussi fortement que possible, afin de limiter les
risques de mouvement latéral en cas de compromission d'un composant.

L'analyse des besoins révèle principalement un usage pour l’hébergement de
services accessible via Internet et donc soumis en permanence à des tentatives
d'intrusion. Des mécanismes d'exception strictement contrôlés restent
disponibles pour les cas qui le requièrent, notamment lorsque l'hôte assume un
rôle de routeur ou de pare-feu réseau.

=== Portabilité

L'interface d'administration et le format de configuration doivent rester
identiques quelle que soit la configuration matérielle ou réseau sous-jacente.
En dehors des dépendances introduites explicitement par l'utilisateur, le
transfert du système d'un environnement à un autre ne doit nécessiter aucune
adaptation de la configuration existante.

Dans un contexte où un même utilisateur administre plusieurs machines aux
profils distincts, cette homogénéité réduit la charge cognitive et rend les
procédures opérationnelles transférables sans friction.

#show heading.where(level: 3): set heading(outlined: true)

== Gestion déclarative

// TODO: référencer Terraform, Docker, et K8s
La gestion et la configuration du système reposent sur le principe de
déclarativité: l'utilisateur décrit les divers éléments de configuration du
système au sein d'un fichier de configuration, téléverse ce fichier sur le
système, et celui-ci tente alors automatiquement et continuellement de faire
converger la configuration fournie avec l'état réel du système. Cette approche
est déjà bien connue dans le contexte de la conteneurisation et de
l'infrastructure, notamment au travers d'outils tels que Terraform, Kubernetes
ou Docker Compose qui adoptent aussi le concept de déclarativité.

// TODO: référencer GitOps et Git (?)
Cette approche présente deux avantages principaux. D'une part, elle permet de
facilement comprendre le système et d'avoir une vue d'ensemble de sa
configuration à tout instant. D'autre part, elle se prête naturellement à une
gestion de type GitOps: le fichier de configuration constituant la source de
vérité unique du système, il suffit de versionner ce fichier dans Git pour
disposer d'un historique complet de l'infrastructure. Appliquer un changement
revient alors simplement à soumettre une nouvelle version du fichier, sans avoir
à se soucier de l'état précédent du système, puisque c'est le contrôleur qui se
charge de calculer et d'exécuter les actions nécessaires pour atteindre le
nouvel état désiré.

// TODO: référencer la théorie du contrôle
Ce principe est directement inspiré de la théorie du contrôle, et plus
précisément des boucles de rétroaction en circuit fermé. Au sein du présent
système, la configuration est décomposée en resources. Celles-ci représente une
interface réseau, une route IP, un disque, un conteneur, etc. Chaque resource
est associée à un contrôleur qui implémente la logique nécessaire afin de la
réconcilier. Le processus-type de réconciliation est illustrée par la boucle de
contrôle présentée dans la @ctrlloop.

#include "../diagrams/ctrlloop.typ"

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

== Modèle de resource
#todo[][
    - Présenter un peu plus les resources (pas forcément exhaustif)
    - Présenter le split user vs dynamic
    - Le concept de manager
]

Chaque resource est uniquement identifiable par la combinaison de son type et de
son nom. A chaque type de resource est associé un schéma propre à celle-ci et
décrivant les éléments propre on domaine concerné. Un état est aussi associé a
chaque resource

La configuration utilisateur est une liste de resources, et cette configuration
n'est modifiable QUE par l'utilisateur. Cette configuration utilisateur donne
lieu a une ou plusieurs resources dynamiques. Par exemple, lors de la
déclaration d'un conteneur, cela donnera lieu, d'une part a une resource "image"
qui téléchargera l'image, et a une resource "conteneur" qui exécutera l'image.

#include "../diagrams/cfgdyn.typ"

// TODO: commenter le schéma ci-dessus

== Interface d'administration

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

== Exemple d'utilisation
#todo[][
    Un exemple simple et concret pour bien cerner
]

= Architecture et sécurité

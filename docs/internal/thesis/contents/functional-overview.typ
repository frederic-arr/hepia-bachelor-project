#import "../lib.typ": *

= Présentation du système <ch:functional-overview>
Ce chapitre présente le système du point de vue de l'utilisateur, sans entrer
dans les détails d'implémentation, et illustre la manière dont les objectifs
présentés dans l'introduction se traduisent concrètement dans son mode
d'administration. Le modèle d'administration général est d'abord introduit,
avant que le modèle de ressources sous-jacent, ainsi que les principes de
déclarativité et d'immutabilité qui en découlent, ne soient détaillés.
L'administration du système via l'API et son client en ligne de commande est
ensuite présentée, suivie des mécanismes d'installation, de la gestion des
conteneurs et des prérequis matériels. Le chapitre se conclut par un exemple
d'utilisation combinant l'ensemble des éléments présentés.

== Vue d'ensemble <ch:functional-overview:overview>
Le système est administré intégralement via une API, à laquelle l'accès est
possible soit directement au moyen d'un client en ligne de commande (CLI), soit
via un provider Terraform, ce dernier étant, par nature, limité à la
configuration du système. En particulier, l'administrateur n'a pas besoin de
télécharger des paquets additional ou de configurer préalablement l'hôte avant
de pouvoir l'administrer et l'utiliser pour y exécuter des conteneurs.
L'ensemble de la configuration est décrit dans un unique fichier au format YAML,
structuré sous la forme de plusieurs documents distincts, séparés par `---`.

Une configuration minimale, définissant un accès à l'API sans authentification
ainsi qu'une interface réseau configurée en DHCP, est présentée dans le
#code-num-ref(<code-config-default>):

#figure(
    label: <code-config-default>,
    caption: [Configuration par défaut],
    note: [
        Configuration par défaut lors du démarrage du système depuis l'image
        ISO.
    ],
    source: made-by-self,
    ```yaml
    ---
    schema: api
    auth: none
    listen: [0.0.0.0/0]
    ---
    schema: network:link
    name: eth0
    admin_up: true
    ---
    schema: network:dhcp
    name: eth0
    ```,
)

Cette configuration, qui est la configuration par défaut lors du démarrage du
système depuis une image ISO, permet de mettre en place le strict minimum pour
que le système soit accessible: l'interface réseau est activée, une
configuration réseau est réceptionnée par DHCP, et l'API est accessible sans
authentification.

Chaque document de ce fichier correspond à la configuration d'une ressource. Au
sein du système, une ressource représente un aspect administrable de celui-ci,
tel qu'une interface réseau, un conteneur ou un client DHCP. L'utilisateur
décrit l'état souhaité pour cet aspect du système, sans se préoccuper de la
manière dont cet état est atteint et le système va automatiquement tenter de
faire réconcilier son état vers l'état désiré.

== Ressources <ch:functional-overview:resources>
Les ressources sont regroupées par domaine fonctionnel, tel que `network`,
`container` ou `system`, puis par type au sein de ce domaine, par exemple
`network:link` pour le lien réseau ou `container:instance` pour une instance de
conteneur. Les ressources peuvent aussi être nommées, cette propriété dépendant
du type: une ressource unique et globale, telle que la configuration DNS, ne
nécessite pas de nom, tandis qu'une ressource pouvant être instanciée plusieurs
fois doit être nommée afin de pouvoir la distinguer des autres instances du même
type. Outre les éléments permettant d'identifier de manière unique une
ressource, le document de configuration contient également la spécification de
la ressource, aussi appelée l'état désiré. Cette spécification contient les
paramètres propres à l'instance spécifique de la ressource, tels que l'image
d'un conteneur, l'état d'une interface réseau, ou les paramètres
d'authentification de l'API.

Il existe ainsi douze ressources, brièvement décrites dans le #table-num-ref(
    <tab-resource-types>,
):

#figure(
    label: <tab-resource-types>,
    caption: [Vue d'ensemble des types de ressources disponibles],
    note: [TODO],
    source: made-by-self,
    table(
        columns: 2,
        align: left,
        ..([*Schéma*], [*Description*]),
        ..([`api`], [Configuration de l'accès à l'API du système]),
        ..([`install`], [Configuration de l'installation sur disque]),
        ..([`system:etc`], [Configuration d'un fichier dans /etc/]),
        ..([`network:dns`], [Configuration de la résolution DNS]),
        ..(
            [`network:dhcp`],
            [Configuration d'un client DHCP sur une interface],
        ),
        ..([`network:link`], [Configuration d'une interface réseau]),
        ..([`network:address`], [Configuration d'une adresse réseau]),
        ..([`network:route`], [Configuration d'une route réseau]),
        ..([`container:runtime`], [Configuration d'un runtime de conteneurs]),
        ..([`container:network`], [Configuration d'un réseau de conteneurs]),
        ..([`container:volume`], [Configuration d'un volume de conteneurs]),
        ..(
            [`container:instance`],
            [Configuration d'une instance de conteneur],
        ),
    ),
)

Ces ressources constituent l'ensemble des aspects configurables du système et
permettent de disposer d'un hôte de conteneurisation à part entière, sans
qu'aucun composant ou service supplémentaire ne soit requis en dehors de ceux
décrits par ces douze types de ressources. Les domaines `network` et `container`
couvrent respectivement l'ensemble de la configuration réseau et l'ensemble du
cycle de vie des conteneurs, tandis que les domaines `api`, `install` et
`system` couvrent l'administration du système lui-même.

== Déclarativité et immutabilité <ch:functional-overview:declarativity>
Le système tentant automatiquement de réconcilier son état avec celui décrit par
l'utilisateur, il s'agit donc d'un système déclaratif. Outre l'avantage de
simplifier l'administration au jour le jour, la déclarativité s'inscrit
particulièrement bien dans un contexte GitOps. Le fichier de configuration
constitue la seule source de vérité du système, toute autre modification étant
généralement impossible. Le système peut néanmoins être amené à réagir à des
événements particuliers, tels que la déconnexion d'un câble réseau ou l'arrêt
inattendu du runtime de conteneurs. Se basant uniquement sur l'état désiré et
sur l'état actuel, et n'effectuant que le plus petit nombre d'actions
nécessaires pour réconcilier ces deux états, le système est en mesure de gérer
automatiquement ce type de situation.

Le système reste, dans son fonctionnement normal, entièrement immuable: la
configuration ne peut être modifiée autrement que par l'API, et les fichiers
nécessaires au démarrage, tels que ceux du répertoire `/bin`, sont stockés et
exposés de manière immuable lorsque le système est en cours d'exécution. À
chaque redémarrage, le système se reconstruit entièrement à partir de ces seuls
fichiers de démarrage et de la configuration, garantissant un état initial
cohérent. Ces contraintes garantissent que l'état du système reste en permanence
traçable et reproductible à partir du seul fichier de configuration.

== Administration du système et API
Le système étant entièrement administré au travers de l'API, il est nécessaire
de fournir un client permettant d'intéragir avec celle-ci. Ce client s'appelle
`cos-cli` et permet d'effectuer les actions essentielles sur le système à
travers divers commandes décrites dans le #table-num-ref(<tab-cli-commands>):

#figure(
    label: <tab-cli-commands>,
    caption: [Commandes disponibles],
    note: [
        La syntaxe est ```sh subsystem command <MANDATORY> [OPTIONAL]```.
    ],
    source: made-by-self,
    table(
        columns: 2,
        align: left,
        ..([*Commande*], [*Description*]),
        ..(
            [
                ```sh config push <PATH>```
            ],
            [upload la config],
        ),
        ..([```sh config pull```], [télécharge la config]),
        ..(
            [```sh fs write <LOCAL_PATH> <VOLUME>:<PATH>```],
            [écrit un fichier sur un volume de conteneur (jamais sur le système
                de fichier racine)],
        ),
        ..([```sh fs list [VOLUME:]<PATH>```], [list les fichiers]),
        ..(
            [```sh fs read [VOLUME:]<PATH>```],
            [télécharger un fichier depuis le server (soit sur un volume, soit
                directement sur le système de fichier racine)],
        ),
        ..(
            [```sh resources list [SCHEMA]```],
            [lister les ressources, éventuellement filtré par type],
        ),
        ..(
            [```sh resources get <SCHEMA> [NAME]```],
            [consulter l'état d'une ressource],
        ),
        ..([```sh system reboot```], [redémarrer ou éteindre le système]),
        ..([```sh container logs```], [consulter les logs d'un conteneur]),
    ),
)

Ce client permet notamment de transmettre une nouvelle configuration au système,
de consulter l'état des ressources existantes, ou d'interagir directement avec
les volumes des conteneurs, sans que l'utilisateur ait à se connecter
directement à la machine hôte.

L'accès à l'API est lui-même régi par une ressource dédiée, au même titre que
toute autre ressource du système, comme illustré dans le #code-num-ref(
    <code-config-default>,
). Cette ressource permet de définir le mécanisme d'authentification requis,
ainsi que l'ensemble des adresses depuis lesquelles l'API demeure accessible,
restreignant ainsi la surface d'exposition du système sur le réseau. L'absence
d'authentification, utilisée dans la configuration par défaut, convient à un
usage de test ou d'évaluation rapide, mais n'est pas recommandée pour un
déploiement exposé à Internet.

== Installation du système et modes de fonctionnement
L'installation du système s'effectue de la même manière que l'administration des
ressources courante: en ajoutant un document d'installation dans la
configuration. Ce document est présenté dans le #code-num-ref(
    <code-config-install>,
):

#figure(
    label: <code-config-install>,
    caption: [Configuration d'installation],
    note: [
        Installation du système entièrement sur le disk /dev/vda, sans
        chiffrement.
    ],
    source: made-by-self,
    ```yaml
    ---
    schema: install
    boot:
        disk: /dev/vda1
    config:
        disk: /dev/vda2
        encryption:
            provider: static
            key: this-is-a-very-secure-password
            autounlock: true
    data:
        disk: /dev/vda3
        encryption:
            provider: tpm2
    ```,
)

Ce document de configuration décrit les trois volumes de stockages sur lequel le
système se repose: le disque de démarrage, contenant les divers artefacts tel
que le noyau, les binaires du système, et le bootloader. Ensuite, le volume
contenant la configuration du système, puis un volume de donné dans lequel
seront stockés les images et les volumes des conteneurs.

Aucun de ces disque n'est obligatoire. Omettre le disque de boot fait qu'il est
toujours nécessaire de disposer d'un support externe afin de démarrer le
système, mais que ce support externe n'a pas besoin de stocker la configuration
ou des données. Omettre le disque de configuration et le disque de données
permet d'avoir un système entièrement éphémère. Chaque redémarrage fourni un
système complètement neuf ce qui peut s'avérer particulièrement pratique dans le
cadre de tests. En outre, dans le cas ou un disque est omis, il est toujours
possible de le rajouter plus tard sans pertes de données.

Les disques supportent le chiffrement à travers un TMP1 ou TMP2, ou à travers
une clef statique. Dans le cas de la clef, celle-ci peut optionellement être
stockés conjointement avec la configuration, au détriment de la sécurité mais
permettant de déverrouiller le volume de manière autonome.

Enfin, il est possible d'omettre complètement le document de configuration,
auquel cas rien ne sera sauvegardé et le système sera restitué dans son état
d'origine après un redémarrage. Dans ce mode de fonctionnement, le système
demeure entièrement utilisable, y compris les volumes de conteneurs, qui seront
stockés en memoire.

== Conteneurs
La conteneurisation repose sur le concept de runtime, qui est simplement
l'interface permettant d'exécuter des conteneurs. Chaque conteneur est associé à
une runtime qui sera en charge de l'administrer. Actuellement, seul Podman est
disponible comme runtime, mais il est toujours utile de pouvoir instancier
plusieurs fois cette runtime, par exemple lorsqu'il est souhaitable de disposer
de container entièrement "rootless". Le #code-num-ref(<code-config-runtime>)
décrit une telle configuration:

#figure(
    label: <code-config-runtime>,
    caption: [Configuration d'une runtime de conteneur],
    note: [
        Une runtime de conteneur utilisant Podman est configurée en mode
        "rootfull".
    ],
    source: made-by-self,
    ```yaml
    ---
    schema: container:runtime
    name: rootfull
    engine: podman
    uid: 0
    gid: 0
    depends_on:
      - network:dns
      - network:route/eth0-dhcp
    ```,
)

Le #code-num-ref(<code-config-runtime>) spécifie l'id utilisateur et groupe
root, mais aussi un ensemble de dépendances à des ressources DNS et réseau. Ces
dépendances ne sont pas strictement nécessaire mais permette d'éviter
d'instantier la runtime tant que la couche réseau n'est pas disponible, ce qui
pourrait avoir pour effet d'empêcher le téléchargement d'images de conteneurs,
et créerait des messages d'erreur temporaire. La nature déclarative du système
décrite dans le #chapter-num-ref(<ch:functional-overview:declarativity>) rend
toutefois ces dépendances optionelles car il est capable de se rétablir seul.

Une fois une runtime configurée, il est possible d'y créer divers ressources tel
que des réseaux de conteneurs ou des instances de conteneurs, comme décrit dans
le #code-num-ref(<code-config-container>):

#figure(
    label: <code-config-container>,
    caption: [Configuration d'un conteneur],
    note: [
        Configuration d'un conteneur qui va requêter une fois un server HTTP
        puis s'arrêter.
    ],
    source: made-by-self,
    ```yaml
    ---
    schema: container:instance
    name: demo
    image: docker.io/alpine/curl:latest
    restart: never
    runtime: rootfull
    cmd: [http://10.0.2.2:1234]
    ```,
)

Le #code-num-ref(<code-config-container>) crée une instance de conteneur nommé
"demo" et ayant pour image la dernière version de cURL sur Alpine Linux. Cette
instance va être démarré, elle va requêter un URL, et si cela réussi, alors le
conteneur s'arrête. Il est aussi possible de supporter des cas plus complexe
grâce aux réseaux de conteneurs et à la publication de ports sur l'hôte via une
syntaxe similaire à Docker Compose. La création de réseau de conteneurs se fait
via une ressource dédiée, illustrée dans le #code-num-ref(
    <code-config-container-network>,
):

#figure(
    label: <code-config-container-network>,
    caption: [Configuration d'un réseau de conteneurs],
    note: [
        Configuration d'un réseau de conteneur sur la runtime de conteneur
        nommée "rootfull".
    ],
    source: made-by-self,
    ```yaml
    ---
    schema: container:network
    name: my-network
    runtime: rootfull
    ```,
)

Tout comme pour la création de conteneurs, il est nécessaire de spécifier la
runtime sur laquelle crée ce réseau, et il est naturellement impossible
d'adjoindre des conteneurs extérieur à cette runtime sur le réseau ainsi créé.

== Prérequis matériels <ch:functional-overview:hardware>
Le système nécessite très peu de ressources: sur un processeur 64 bits, il est
possible de démarrer un server web minimale avec seulement 170 MiB de mémoire
vive. Il est même possible de démarrer le système sur moins de 90 MiB de RAM si
les aspects liés à la conteneurisation ne sont pas nécessaire, par exemple afin
de fournir un routeur rudimentaire. En outre, dans le cadre d'une installation
complète, le système ne nécessite que 1 GiB de stockage.

== Exemple d'utilisation <ch:functional-overview:example>
Le scénario suivant illustre l'ensemble des éléments présentés dans ce chapitre:
une instance de machine virtuelle, fraîchement créée chez un fournisseur cloud
et démarrée depuis l'image ISO du système, est configurée, installée sur son
disque, puis déployée avec un serveur HTTP retournant le message "Hello,
world!".

L'ensemble des ressources nécessaires à ce scénario est combiné au sein d'un
unique fichier de configuration, versionnable dans un dépôt Git. Ce fichier doit
se situer sur la machine depuis laquelle le client `cos-cli` est exécuté. Le
#appendix-num-ref(<appendix-full-config>) illustre une telle configuration,
combinant l'installation du système sur disque, l'accès à l'API, la
configuration réseau, ainsi qu'un conteneur exécutant un serveur HTTP minimal.

Une fois la machine virtuelle démarrée depuis l'image ISO, la commande suivante
permet d'appliquer cette configuration au serveur dont l'adresse est désignée
par `$IP`; elle déclenche à la fois l'installation du système sur le disque de
l'instance et le déploiement du serveur HTTP:

#figure(
    label: <cmd-install>,
    caption: [Commande d'installation],
    note: [
        La commande est exécutée depuis un poste distinct du serveur cible,
        disposant d'un accès réseau à celui-ci; `$IP` désigne l'adresse de ce
        serveur.
    ],
    source: made-by-self,
    ```sh
    $ cos-cli --server $IP config push ./config.yaml
    ```,
)

Le serveur HTTP est accessible depuis l'adresse `$IP` et le port 80 de
l'instance moins de 30 secondes après l'exécution de la commande précédente,
cette durée étant mesurée et détaillée au #chapter-full-ref(
    <ch:validation:speed>,
). Aucune configuration supplémentaire n'est nécessaire sur le fournisseur cloud
au-delà de l'ouverture du port correspondant. La même commande est réutilisée
pour toute mise à jour ultérieure de la configuration, par exemple pour modifier
l'image du conteneur ou ajouter un second service. Le système se charge de
réconcilier automatiquement l'état effectif avec la nouvelle configuration
transmise, sans redémarrage ni interruption des ressources déjà en place,
conformément au modèle de réconciliation présenté au #chapter-num-ref(
    <ch:functional-overview:declarativity>,
).

Le même processus est également accessible via Terraform, dans le cas où
l'instance cloud et le déploiement du système sont administrés au sein d'une
même infrastructure:

#figure(
    label: <code-terraform>,
    caption: [Administration via Terraform],
    note: [TODO],
    source: made-by-self,
    ```terraform
    resource "cos_push_config" "my_server" {
        server = some_cloud_provider.vm.ip
        config = file("./config.yaml")
    }
    ```,
)

/*
Le système permet également de configurer des conteneurs privilégiés, capables
d'interagir directement avec le système et, potentiellement, de le modifier.
Dans ce cas, il n'est pas possible de garantir que le conteneur ne modifie pas
l'état du système d'une manière qui le rendrait irréconciliable. Le système
reste, dans le cas général, immuable: la configuration ne peut être modifiée
autrement que par l'API, et les fichiers nécessaires au démarrage, tels que ceux
du répertoire `/bin`, sont stockés et exposés de manière immuable lorsque le
système est en cours d'exécution. À chaque redémarrage, le système se
reconstruit entièrement à partir de ces seuls fichiers de démarrage et de la
configuration, garantissant un état initial cohérent, tant que l'intégrité de
ces fichiers eux-mêmes n'est pas compromise.

Cette garantie d'immutabilité cesse en effet de s'appliquer dès qu'un conteneur
privilégié dispose d'un accès direct à un périphérique bloc sous-jacent, tel que
le disque de démarrage. Un tel accès permet d'écrire directement sur la
partition contenant les fichiers de démarrage immuables, en dehors de tout
contrôle exercé par le système de fichiers en cours d'exécution. Ce cas ne
constitue donc pas une simple exception marginale au modèle d'immutabilité, mais
une limite structurelle de celui-ci: l'octroi d'un accès bloc bas niveau à un
conteneur revient à lui accorder un contrôle équivalent à celui d'un accès
physique à la machine.

Ces contraintes garantissent que l'état du système reste, dans la grande
majorité des cas, traçable et reproductible à partir du seul fichier de
configuration #footnote[
    L'exception principale à cette garantie concerne les conteneurs privilégiés
    disposant d'un accès direct à un périphérique bloc; un tel accès doit donc
    être accordé avec la même prudence qu'un accès physique à la machine.
].
*/

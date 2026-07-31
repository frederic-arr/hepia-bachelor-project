#import "../lib.typ": *

= Comparaison avec les solutions existantes

== Empreinte mémoire

La #figure-num-ref(<val-memory>) présentée au chapitre précédent établit une
allocation mémoire médiane de 208~MiB pour un conteneur exécuté sur une VM
disposant de 256~MiB de RAM, ce qui correspond à une consommation propre au
système d'exploitation d'environ 20~MiB une fois le conteneur démarré.

Une mesure équivalente n'a pas été reproduite sur NixOS ni sur Talos Linux, en
raison de la charge de travail que représenterait l'instrumentation de ces deux
systèmes selon un protocole strictement comparable. La documentation officielle
de Talos Linux rapporte une consommation mémoire au repos de l'ordre de
300~à~500~MiB pour un nœud fraîchement démarré, soit un ordre de grandeur
supérieur d'un facteur 15 à 25 par rapport au système développé. Aucune donnée
équivalente n'est disponible pour NixOS, la consommation mémoire d'une
installation NixOS dépendant fortement de l'ensemble de services activés dans la
configuration, ce qui rend toute comparaison ponctuelle peu représentative sans
protocole de mesure dédié.

Cette différence d'ordre de grandeur s'explique en partie par la vocation de
Talos Linux, conçu pour exécuter un nœud Kubernetes complet, incluant `kubelet`,
`containerd` et les composants du plan de contrôle, alors que le système
développé se limite à l'exécution d'un runtime de conteneur unique.

== Taille de l'image de démarrage

L'image ISO du système développé occupe 261~MiB et inclut l'ensemble des
composants nécessaires à l'exécution du système, sans téléchargement additionnel
requis au démarrage.

Les images ISO minimales de NixOS distribuées officiellement occupent environ
1,5~GiB, soit un facteur supérieur à 5 par rapport au système développé; cette
taille s'explique notamment par l'inclusion du paquet `linux-firmware`,
représentant à lui seul une part importante du volume compressé. La taille
exacte de l'ISO officielle de Talos Linux n'a pas pu être établie avec certitude
à partir des sources consultées et devrait être mesurée directement à partir
d'une image téléchargée depuis Image Factory avant intégration définitive de
cette comparaison.

== Nombre de binaires embarqués

L'image du système développé inclut 260 binaires. Talos Linux revendique
l'absence de shell interactif, de gestionnaire de paquets et de SSH, ainsi qu'un
nombre de binaires embarqués nettement inférieur à celui d'une distribution
Linux généraliste. Cette caractéristique constitue un critère de comparaison
qualitatif pertinent, la réduction du nombre de binaires exécutables diminuant
la surface d'attaque du système. Une mesure exacte et directement comparable
devrait être effectuée sur une installation Talos Linux de référence afin de
fonder cette comparaison sur des valeurs mesurées plutôt que déclaratives.

== Workflow d'installation

Le système développé repose sur une configuration déclarative unique, structurée
en documents YAML distincts par domaine fonctionnel (installation,
authentification, réseau, conteneurs), appliquée en une seule opération lors du
démarrage initial.

Le workflow d'installation de Talos Linux repose sur la génération d'une
configuration via l'outil `talosctl`, suivie de son application à un nœud
démarré depuis une image ISO ou une image disque, puis d'une étape de bootstrap
distincte pour l'initialisation du plan de contrôle Kubernetes. Le workflow de
NixOS repose quant à lui sur la rédaction d'un fichier de configuration Nix,
suivi d'une phase de build locale ou distante avant application au système
cible. Contrairement à ces deux approches, le système développé ne nécessite ni
étape de build préalable côté utilisateur, ni étape de bootstrap distincte, la
configuration étant appliquée directement au premier démarrage.

== Taille du fichier de configuration

La configuration minimale du système développé, incluant l'installation,
l'authentification, le réseau et le déploiement d'un conteneur, occupe moins de
40 lignes au format YAML.

Une configuration Talos Linux générée par `talosctl gen config` occupe
typiquement plusieurs centaines de lignes par nœud, incluant les certificats,
les clés et les paramètres du plan de contrôle. Une configuration NixOS minimale
occupe généralement entre 20 et 50 lignes pour un système de base, mais croît
rapidement avec l'ajout de modules et de services. Cette comparaison doit être
interprétée avec prudence méthodologique, la complexité fonctionnelle couverte
par chaque configuration n'étant pas strictement équivalente entre les trois
systèmes.

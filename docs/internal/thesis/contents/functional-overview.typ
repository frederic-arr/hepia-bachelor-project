#import "../lib.typ": *

= Présentation du système

#todo-note[
    - Présente le système pour un utilisateur.
    - Mélange un petit peu un "Quick Start" / "Getting Started" et un "Learn
        More" de haut niveau
    - Volume estimé: ~10-13 vrai pages
]

#todo-inline[Amorce du chapitre + plan interne + synthèse brève.]

#todo-inline[Référencer les divers outils.]

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

== Ressources

=== Définition <simple-res-def>
L'administration et la configuration du système est organisé autour des
ressources. Une ressource est simplement l'abstraction d'un ou plusieurs objets
concrets au sein du système. Par exemple, la ressource "conteneur" représente le
concept du même nom; la ressource "interface réseau" représente quant à elle un
lien réseau, une ou plusieurs routes, et une ou plusieurs addresses.

Une ressource contient quatre informations principales:
+ un type, par exemple "interface réseau", "conteneur", "volume", etc.
+ un nom arbitraire, qui permet d'identifier uniquement plusieurs ressources du
    même type.
+ une spécification, qui représente les divers options que l'administrateur
    souhaite paramétrer sur la ressource, par exemple l'image d'un conteneur,
    l'addresse IP d'une interface réseau, etc.
+ un état, qui reflète simplement les informations actuelles de l'objet
    sous-jacent. Par exemple, pour un conteneur, cela peut contenir l'état
    actuel tel que "running" ou "stopped", le message d'erreur ayant empêché de
    démarrer le conteneur, etc.

Les ressources dont l'utilisateur peut directement modifier la spécification
sont appelés ressources statiques. Cela signifie que le seul moyen de les mettre
à jour est de passer explicitement par l'interface d'administration afin de
téléverser une nouvelle version de la configuration du système. La configuration
du système est simplement une liste l'ensemble des ressources que
l'administrateur gère directement, avec leur type, leur nom, et leur
spécification. Chaque ressource statique configuré par l'utilisateur va crée une
sous-ressource dite dynamique. Cela permet de masquer les détails et la
complexité de certaines ressources derrière une interface simple, par exemple en
agrégeant les divers objets de réseau derrière une seule configuration. La
création de ressources dynamique n'est par ailleur pas limité uniquement à la
configuration utilisateur: une ressource dynamique peut créer d'autres
ressources dynamique. À partir d'une ressource dynamique, il est toujours
possible de remonter la chaîne jusqu'à trouver la configuration utilisateur
ayant _in-fine_ causé la création de cette ressource Un exemple typique d'une
aggregation de ressource est illustré dans la @cfgdyn.

#include "../diagrams/cfgdyn.typ"

Dans le cas d'une configuration réseau, il est plus naturel de penser
l'interface, l'adresse et le routage comme une seule entité~#bref(<cfgdyn-cfg>),
bien que ce soient en réalité trois entités distinctes~#bref(<cfgdyn-dyn>).
L'administrateur ne peut pas modifier directement ces resources dynamiques.
Celles-ci sont créées à la volée par le contrôleur et peuvent être ajustées
automatiquement par le système (par exemple, si l'interface est configurée en
DHCP, la resource d'adresse est mise à jour régulièrement). Le seul levier
d'action de l'administrateur est de modifier la configuration, qui est la racine
de toutes les autres resources.

=== Réseau
#todo[Firewall, VPN, DHCP, IP statique, IPv6, etc.]

=== Conteneurisation
#todo[Conteneur, gestion des images, registres privé, Podman, Secrets, Limites]

=== Stockage et volumes
#todo[
    Partitionnement disque, volume, cache, data, chiffrement (on peut combiner
    plusieurs methodes de chiffrement!)
]

=== Portabilité de la configuration
#todo[Portabilité][
    - On peut matcher une interface par une MAC address (partielle), nom, index,
        etc.
    - Idem pour les disques
    - En tous les cas, si une config a trois interface réseau mais que
        physiquement il y en a que une sur la nouvelle machine, pas de magie
]

== Réconciliation
=== Déclarativité et boucle de contrôle <simple-ctrlloop>
Le processus permettant de répercuter les modification apportée la spécification
d'une ressource sur l'objet physique sous-jacent s'appel la réconciliation. À
chaque réconciliation d'une ressource, le système va renouveller l'état actuel
de la ressource, le comparer avec la spécification, puis effectuer les actions
correctives pour faire converger l'état actuel avec la spécification tel
qu'illustré dans la @decl.

#include "../diagrams/decl.typ"

Par exemple, dans le cas de la configuration d'une interface réseau, il est
spécifié que son status doit être "up", cela constitue donc l'état désiré~#bref(
    <decl-cfg>,
). L'état actuel de l'interface (statut, adresse IP, etc.) est d'abord
récupéré~#bref(
    <decl-obs>,
), puis comparé à l'état désiré désiré~#bref(<decl-diff>). Dans le cas ou le
status serait "down", le système s'en rend compte et sait qu'il doit exécuter
l'équivalent de `ip link set up` afin de mettre en route l'interface~#bref(
    <decl-actions>,
). Ce même mécanisme s'applique à toute modification ou suppression de la
ressource: une modification de la configuration déclarative se traduit
automatiquement par les actions correctives adéquates.

Le système prend séquentiellement chaque ressource et applique cette logique de
réconciliation, une fois arrivé à la dernière ressource, il recommence, tel
qu'illustré dans la @ctrlloop.

#include "../diagrams/ctrlloop.typ"

En intégrant la réconciliation à une boucle, cela permet de corriger les dérives
du système. Par exemple si un câble réseau est débranché puis rebranché, le
système s'en rendra compte a la prochaine réconciliation de la ressource
correspondante et effectuera les actions nécessaire.

=== Gestion des erreurs
Ce même mécanisme permet aussi de gérer les erreur transitive de manière
efficaces: lorsqu'un tel événement survient, la boucle va simplement passer a la
ressource suivante. À la prochaine itération de la boucle, le système réessayera
de réconcilier la ressources. Le système expose l'erreur dans l'état actuel de
la ressource, permettant à l'administrateur de diagnostiquer la source du
problème, en particulier lorsque l'erreur est persistante. En outre, le système
garde un historique des changements d'états de la ressource afin de permettre le
diagnostic de problèmes plus complexe.

=== Gestion des dépendances
Certaines ressources dépendent d'autre ressources, c'est par exemple le cas
lorsqu'un conteneur est assigné à un réseau de conteneurs. Dans ce cas, le
système s'efforce d'effectuer la réconciliation dans l'ordre des dépendances. Si
malgrès tout, au moment d'une réconciliation, la dépendance n'est pas encore
prête, cela est simplement traité comme une erreur transitive, qui se résoudra
éventuellement, suivant le processus de gestion d'erreur.

#todo[Dépendances cycliques]

== Administration du système
=== Exigences matérielles
#todo[Hardware requirements][
    - RAM
    - Storage
]

=== Interface d'administration
#todo[Interface d'administration][
    - Parler de la CLI avec un tableau des commandes les plus importantes
    - Parler de Terraform vite fait
    - Parler de comment on s'authentifie a l'interface (token, mTLS, etc.)
    - Parler des commandes importantes
        - Push config
        - Voir l'état
        - Téléverser des fichiers
        - Diagnostic réseau
        - exec dans un conteneurs
        - les logs
        - reboot
]

=== Sources de configuration
#todo[Config src][
    - disk
    - remote
    - cloud-init
    - etc.
    - Schéma des sources
]

=== Monitoring et observabilité
#todo[Monitoring][
    - En gros logs accessible via le CLI, ou exporté via les trucs standard
        (Loki, ES, etc.)
    - Pour l'observabilité, idem + exporté via Prometheus
    - Export des métriques et logs host *ET* conteneurs
]

=== Persistence des données
#todo[Persistence des données][
    - Comment consulter le stockage
    - Parler du fait que le disque est auto-déverrouillé + risques que cela
        implique
    - Parler en particulier du risque du mode "passphrase" pour le chiffrement
    - On peut aussi utiliser une passphrase en mode manuel, dans ce cas le
        système n'est pas auto-déverrouillé
]

== Cycle de vie du système
=== Personnalisation de l'image
#todo[Personnalisation][
    - Comment intégrer de nouvelles options du noyau
    - De nouveau drivers
    - Pre-bake une image conteneur ou une config
    - etc
]
=== Méthodes de démarrage
#todo[Boot][
    - boot en réseau
    - mode in-memory
    - différent bootloader
]

=== Installation initiale
#todo[
    Parler du fait qu'il suffit simplement de push une configuration valide sur
    un système vierge en maintenance.
]

=== Mise à jour et retours en arrière
#todo[
    - Parler du mécanisme de mise à jour du système avec le champ `osVersion` et
        l'installation A/B.
    - Parler du fait qu'on peut rollback la config ou la version du système (ou
        les deux).
]

=== Maintenance
#todo[
    Parler du mode de maintenance et du mode d'authentication additionnel propre
    à celui-ci.
]

=== Sauvegarde et restauration
#todo[
    - Parler de comment backup/restore le système
    - Comment exporter un volume/conteneur individuellement aussi
]

== Exemple d'utilisation
#todo[Deployer un conteneur simple avec un serveur HTTP]

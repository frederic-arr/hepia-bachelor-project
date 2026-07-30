#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq

= Tests et validation

== Tests unitaires et tests d'intégration

Les tests unitaires et les tests d'intégration portent principalement sur les
contrôleurs. La logique de validation des ressources est systématiquement
couverte par des tests unitaires. Les tests d'intégration couvrent
systématiquement les cas suivants: la création d'une nouvelle ressource, la mise
à jour d'une ressource existante, la tentative de gestion d'une ressource
inexistante (par exemple une interface réseau absente), la tentative de gestion
d'une ressource déjà gérée par un autre composant, ainsi que la suppression
d'une ressource. Seul une partie des contrôleurs sont testés via des tests
d'intégration. En effet, l'environement dans lequel les tests sont isolé ne
dispose pas de tous les éléments nécessaires pour mener a bien le test de
certains composants. C'est nottemetn le cas du contrôleur de conteneurs:
l'environement de test disposant d'une système de fichier raçine vide, la
runtime de conteneur n'est pas présente et celle-ci nécessite un nombre
important de dépendance qu'il serait fastidieux de lier dans cet environement.

Toutefois, des tests de bout en bout (end-to-end, E2E) viennent complèter la
couverture. Ces tests reposent fortement sur Nix afin de build le système
d'exploitation et crée une image ISO. Chaque test est effectué dans une machine
virtuelle séparée. De manière générale, le test va lancer la VM, attendre que
l'API soit joignable puis effectuer une suite de commandes. Afin de garantir que
le système fonctionne correctement, les tests ne reposent pas uniquement sur la
lecture de l'état courrant de l'API, mais incluent dans la configuration un
conteneur `cURL` qui va effectuer une requête HTTP vers l'hôte de test, sur un
port que le test écoute. Cela permet de valider que, non seulement l'API
retourne un état cohérent, mais que celui-ci reflète l'état réel.

== Validation

Parmis les tests de bout en bout, trois test notable existent: l'exécution d'un
conteneur dans un environement ephémère, l'installation du système, et
l'installation d'une application 3 tier classique.

=== Exécution dans un environement ephémère

// TODO: Pareil que le benchmark de rapidité d'exécution

=== Installation du système

// TODO: Pareil que le benchmark de rapidité d'installation

=== Application 3 tier

#todo-missing[]

== Validation & Benchmarking

=== Rapidité

La #figure-num-ref(<val-boot-time-noinstall>) présente la distribution du temps
de démarrage de l'OS, mesuré entre le lancement de le lancement du noyeau par le
bootloader et la réception d'une route via DHCP, moment à partir duquel l'API
devient accessible.

#include "../diagrams/val-boot-time-noinstall.typ"

Au total, environ 1.5s s'écoulent entre le démarrage du noyeau et le moment ou
l'API devient joignable (en orange). La majorité de ce temps (\~1s, en bleu) est
passé dans l'initialisation du noyeau, les 0.5s restantes (en rouge) étant liée
à la réconciliation et au protocol DHCP.

De même, la #figure-num-ref(<val-container-time-noinstall>) présente le temps de
démarrage d'un conteneur, mesuré entre la soumission d'une configuration créant
le conteneur et la réception d'une requête sur un port arbitraire de l'hôte,
émise par ce conteneur.

#include "../diagrams/val-container-time-noinstall.typ"

Entre le démarrage du noyeau et la réception de la requête du conteneur, 5s
s'écoulent (en orange), la majorité du temps, environ 2.3s (en rouge), est passé
à télécharger l'image, ce qui surivent un peu moins de 3s après le démarrage du
noyeau (en blue).

#todo[Validation  mémoire installation][
    - Test pas 100% représentatif de la réalité. il peut y avoir du délai DHCP,
        surcharge CPU, délai réseau, etc. Cela représente ici le meilleur cas
        (modulo le téléchargement)
]

=== Légèreté

Une machine virtuelle disposant de 256~MiB de RAM est utilisée pour ce test. Une
configuration minimale est appliquée en mode éphémère, définissant un conteneur
nommé chargé d'exécuter en boucle la séquence suivante: allocation d'un vecteur,
écriture d'une suite de valeurs dans ce vecteur, vérification de la suite, envoi
de la taille d'allocation courante via un socket, désallocation, puis
incrémentation de l'allocation d'un mégaoctet. Cette boucle se poursuit jusqu'à
l'échec de l'allocation ou de la vérification, ce qui permet de déterminer la
quantité de mémoire effectivement disponible pour un conteneur une fois le
système démarré.

La #figure-num-ref(<val-memory>) présente la distribution de l'allocation
mémoire maximale atteinte par le conteneur "membench" sur cent exécutions, une
fois le système démarré avec une VM disposant de 256~MiB de RAM.

#include "../diagrams/val-memory.typ"

La médiane de l'allocation atteinte se situe à environ 208~MiB, l'intervalle
interquartile s'étendant de 205~à~209~MiB. Les valeurs extrêmes observées se
situent entre 193~et~213 MiB. La mémoire disponible est donc autour des 80% de
la mémoire allouée à la VM. Par extension, le système d'exploitation complet ne
consomme donc qu'environ 20~MiB. Toutefois, il n'est pas pour autant possible de
démarrer une VM avec seulemetn 20~MiB de mémoire. En effet, durant le démarrage,
un minimum de 90~MiB sont requis afin que le système démarre, et dans l'optique
de télécharger une image et exécuter un conteneur au minimum 164~MiB sont
requis.

Enfin, l'image ISO final occupe 261~MiB d'espace disque et inclut l'ensemble du
système sans besoin de téléchargement additionel. Elle inclut 260 binaires.

== Limitations
#todo[Limitation validation][
    - Test pas 100% représentatif de la réalité. il peut y avoir du délai DHCP,
        surcharge CPU, délai réseau, etc. Cela représente ici le meilleur cas
        (modulo le téléchargement)
]

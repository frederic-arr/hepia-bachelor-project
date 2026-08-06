#import "../lib.typ": *

#let vl(v) = box(
    [],
    height: 6pt,
    width: 10pt,
    fill: v.transparentize(75%),
    stroke: v,
)

= Tests et validation
Ce chapitre présente la démarche de validation adoptée pour vérifier le
comportement du système implémenté. La couverture assurée par les tests
unitaires et les tests d'intégration, propres à chaque composant, est d'abord
présentée, avant que les tests de bout en bout, reproduisant des scénarios
d'utilisation complets du système, ne soient détaillés. Ces derniers servent
également de base à l'analyse de performance, qui mesure la rapidité de
démarrage et d'installation du système ainsi que son empreinte mémoire. Le
chapitre se conclut par une discussion des limites du protocole de mesure
employé.

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
certains composants. C'est notamment le cas du contrôleur de conteneurs:
l'environement de test disposant d'une système de fichier racine vide, le
runtime de conteneur n'est pas présent et celui-ci nécessite un nombre important
de dépendance qu'il serait fastidieux de lier dans cet environement.

Toutefois, des tests de bout en bout (end-to-end, E2E), implémentés dans #repo(
    "rust/e2e/tests/",
) viennent completer la couverture. Ces tests reposent fortement sur Nix afin
crée les artefacts permettant d'exécuter une machine virtuelle, en particulier
une image ISO. Chaque test est effectué dans une machine virtuelle séparée, tel
que présenté dans le #chapter-full-ref(<ch:implementation:tests>). De manière
générale, le test va lancer la machine virtuelle, attendre que l'API soit
joignable puis effectuer une suite de commandes. Afin de garantir que le système
fonctionne correctement, les tests ne reposent pas uniquement sur la lecture de
l'état courrant de l'API, mais incluent dans la configuration un conteneur cURL
qui va effectuer une requête HTTP vers l'hôte de test, sur un port que le test
écoute. La réception de cette requête permet ainsi de valider que l'état
rapporté par l'API ne se contente pas d'être cohérent en apparence, mais reflète
bien l'état réel du système et démontrant que le conteneur fonctionne.

== Validation
Parmi les tests de bout en bout, trois scénarios notables sont définis:
l'exécution d'un conteneur dans un environnement éphémère, l'installation du
système suivi de l'exécution d'un conteneur, et le déploiement d'une application
3 tiers.

=== Exécution dans un environnement éphémère
La configuration est appliquée directement sur le système démarré depuis l'image
ISO, sans écriture sur disque, et définit un unique conteneur exécutant une
requête HTTP vers un service exposé par l'hôte de test. La validation consiste à
vérifier que l'état rapporté par l'API à l'issue du cycle de démarrage
correspond à l'état attendu (conteneur en cours d'exécution), et que la requête
HTTP émise par le conteneur est effectivement reçue par l'hôte de test.

=== Installation du système
Le système est installé sur le disque de la machine virtuelle à partir de
l'image ISO, puis redémarré. La configuration appliquée définit également un
conteneur, dont l'exécution est vérifiée selon les mêmes critères que ceux du
scénario précédent. La validation porte en outre sur la persistance de l'état à
travers le redémarrage: l'état de la configuration et du conteneur, tel que
rapporté par l'API après redémarrage, doit correspondre à l'état appliqué avant
redémarrage.

=== Application 3 tiers
// TODO: Nextcloud
Ce scénario met en œuvre une configuration composée de quatre conteneurs: une
base de données, un service backend dépendant de la base de données, un service
web dépendant du backend, et un conteneur de "probe" dépendant du service web,
chargé d'émettre une requête HTTP l'hôte.

Lorsque l'hôte reçoit la requête HTTP, le test va alors initier une requête sur
le conteneur "web" qui va transmettre celle-ci au conteneur "backend", puis la
persister sur la base de données. Le résultat est ensuite vérifié, puis la
machine redémarré puis le résultat revérifié afin de valider que la données a
bien été persisté.

== Analyse de performance <ch:validation:bench>
Les scénarios d'exécution en environnement éphémère et d'installation du
système, décrits à la section précédente, sont repris ici selon le même
protocole, en y ajoutant une instrumentation permettant de mesurer le temps
écoulé entre chaque étape du cycle de vie, ainsi que la mémoire consommée par le
système, le tout sur 100 échantillon.

Les mesures reposent sur la configuration QEMU présentée au~#code-num-ref(
    <code-qemu-bench>,
), exécutée sur un hôte doté d'un processeur AMD Ryzen 7 7700X, de 64 GiB de
mémoire vive et d'un support de stockage NVMe. Cet hôte fonctionne sous Windows
11, les tests étant effectués au sein d'un environnement WSL 2 exécutant Debian.

#figure(
    label: <code-qemu-bench>,
    caption: [Commande QEMU utilisée pour les mesures de performance],
    source: made-by-self,
    ```sh
    qemu-system-x86_64 -cdrom result \
        -drive file=disk.img,format=raw,if=virtio \
        -enable-kvm \
        -cpu host -m 256M \
        -netdev user,id=net0,hostfwd=tcp::50000-:50000 \
        -device e1000,netdev=net0 \
        -nographic
    ```,
)

La majorité des instants mesurés sont déterminés par horodatage, côté tests, des
messages émis sur la console par les différents composants du système, cette
dernière étant flushée immédiatement après chaque message afin de garantir la
fidélité de l'horodatage par rapport à l'instant d'émission. La marge d'erreur
associée à ce mécanisme est jugée négligeable au regard de l'échelle de temps
mesurées. Le démarrage d'un conteneur constitue une exception à ce mécanisme:
cette mesure repose sur le même protocole que les tests de bout en bout décrits
précédemment, à savoir la réception, par un serveur à l'écoute sur l'hôte, d'une
requête HTTP émise par le conteneur concerné.

=== Rapidité <ch:validation:speed>
La #figure-num-ref(<val-boot-time>) présente la chronologie des étapes de
démarrage jusqu'à l'exécution d'un conteneur, pour deux modes de démarrage: une
installation préalable sur disque (plan supérieur) et un démarrage éphémère
depuis l'image ISO, sans installation (plan inférieur). Cinq instants sont
mesurés depuis le démarrage du noyau par le bootloader: le passage à `/init`
("Time until /init", en vert~#vl(green)), la réception d'une route via DHCP
("Time until DHCP route received", en bleu~#vl(blue)), moment à partir duquel
l'API devient accessible, le début du téléchargement d'une image de conteneur
("Time until image downloading", en turquoise~#vl(teal)), le démarrage d'un
conteneur dont l'image est déjà présente localement ("Time until container
started (no pull)", en violet~#vl(purple)), et le démarrage d'un conteneur dont
l'image doit être téléchargée ("Time until container started (pull)", en
rouge~#vl(red)):

#include "../diagrams/val-boot-time.typ"

Dans les deux modes de démarrage, un peu moins d'une seconde s'écoule entre le
démarrage du noyau et le passage à `/init`, puis environ 0.6 secondes
supplémentaire est nécessaire à la réconciliation et au protocole DHCP, portant
à environ 1.5 secondes le délai avant que l'API ne devienne joignable.

Le téléchargement de l'image du conteneur, s'il y a lieu, commence environ 0.5
secondes après la configuration DHCP. Une fois celui-ci commencé, environ 2.1
secondes sont nécessaires pour qu'il arrive à son terme. Le conteneur est
immédiatement démarré une fois ce téléchargement terminé . Lorsque l'image est
déjà téléchargée, le téléchargement se termine instantanément et le conteneur
est aussitôt démarré, ce qui crée une superposition des deux événements sur la
#figure-num-ref(<val-boot-time>). Le cas d'une image déjà téléchargée n'est, par
nature, pas possible pour un environnement éphémère et n'est donc pas représenté
sur le plan inférieur.

Au total, entre le démarrage de la machine et le démarrage du conteneur, le
temps médian est de 5.1 secondes dans le cas d'un téléchargement d'image, contre
environ 2.1 secondes lorsque l'image est déjà présente localement. En excluant
les délais qui ne relèvent pas directement du système, à savoir le chargement du
noyau, l'obtention d'une configuration réseau via DHCP et le téléchargement de
l'image, le temps propre au démarrage du conteneur est inférieur à 500
millisecondes. Il n'y a par ailleur pas différence notable entre un démarrage
sur disque et un démarrage depuis l'image ISO.

La #figure-num-ref(<val-install-time>) présente la durée du processus
d'installation ("Time to install", en bleu~#vl(blue)), ainsi que la durée totale
jusqu'au démarrage d'un conteneur après installation ("Time until container
started", en orange~#vl(orange)). La première mesure l'intervalle entre la
réception de la configuration et la fin de l'écriture des artefacts sur le
disque cible, avant redémarrage. La seconde mesure l'intervalle entre ce même
instant de référence et le démarrage du conteneur, incluant le redémarrage du
système, la réconciliation réseau et le téléchargement de l'image.

#include "../diagrams/val-install-time.typ"

L'installation proprement dite se conclut en environ 6.3 secondes. Le démarrage
complet du conteneur, incluant le redémarrage du système et le cycle décrit à la
#figure-num-ref(<val-boot-time>), se conclut quant à lui en environ 19.3
secondes. Ce total inclus le temps nécessaire à l'hyperviseur pour redémarrer la
machine virtuelle (par exemple le chargement du BIOS) et le délais de sélection
du bootloader (environ 5 secondes).

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
mémoire maximale atteinte par le conteneur sur cent exécutions, une fois le
système démarré.

#include "../diagrams/val-memory.typ"

La médiane de l'allocation atteinte se situe à environ 208~MiB. Les valeurs
extrêmes observées se situent entre 193~et~213 MiB. La mémoire disponible est
donc autour des~80%~de la mémoire allouée à la machine virtuelle. Par extension,
le système d'exploitation complet, noyau et runtime de conteneur inclus, ne
consomme donc qu'environ 40~MiB. Toutefois, il n'est pas pour autant possible de
démarrer une machine virtuelle avec seulement 20~MiB de mémoire. En effet,
durant le démarrage, un minimum de 80~MiB sont requis afin que le système
démarre, et dans l'optique de télécharger une image et exécuter un conteneur au
minimum 160~MiB sont requis.

== Limitations
Le protocole de mesure employé pour les benchmarks de rapidité et de légèreté ne
reflète pas nécessairement l'ensemble des conditions rencontrées en usage réel.
Les mesures présentées correspondent à un scénario favorable, dans lequel le
délai d'obtention d'une adresse via DHCP, la charge du processeur hôte et la
latence réseau ne sont pas artificiellement dégradés. Une charge processeur ou
un délai réseau plus élevés que ceux observés durant les mesures conduiraient à
une augmentation des temps rapportés, notamment pour les étapes dépendant du
réseau, telles que la réconciliation DHCP et le téléchargement d'image.

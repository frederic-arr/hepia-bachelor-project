#import "../lib.typ": *

= Conception du système
Ce chapitre présente l'architecture conceptuelle du système, en explicitant les
décisions architecturales importantes. Les composants fondamentaux du système
sont d'abord brièvement introduits, avant que le modèle de ressource ne soit
détaillé, notamment sa structure, les relations qui lient les ressources entre
elles, ainsi que les différentes catégories. L'orchestrateur et le mécanisme de
réconciliation sont ensuite présentés, en couvrant le rôle du contrôleur, le
choix d'une architecture d'orchestration centralisée, la planification des
cycles de réconciliation, la suppression des ressources, ainsi que la réaction
aux événements internes et externes qui peuvent survenir en dehors de ce cycle.

== Composants du système
Le système repose sur quatre notions fondamentales, détaillées dans le reste de
ce chapitre. Une ressource représente un élément administrable du système,
décrit par une spécification correspondant à l'état désiré. L'orchestrateur est
le composant central chargé de stocker l'ensemble des ressources, d'ordonnancer
leur réconciliation et de maintenir la cohérence des relations qui les lient. Un
contrôleur implémente, pour un ou plusieurs types de ressources, la logique
permettant de faire converger l'état observé vers la spécification. Enfin, la
réconciliation désigne le processus, exécuté de manière continue, par lequel un
contrôleur observe l'état réel d'une ressource, le compare à sa spécification,
et produit les actions correctives nécessaires à leur convergence.

== Modèle de ressource

=== Structure d'une ressource <ch:system-design:struct>
Toute ressource partage une structure commune composée d'un type, d'un nom,
d'une spécification, d'un état observé représentant l'état réel à un instant
$T$, d'un statut et d'une phase. Le type fixe le schéma structurel de la
spécification et de l'état observé, tandis que le nom identifie de manière
unique une instance au sein de ce type. La spécification constitue l'état
désiré, tandis que l'état observé reflète la réalité constatée par le
contrôleur; l'état observé est réévalué à chaque cycle de réconciliation tel que
détaillé dans le #chapter-full-ref(<ch:system-design:declarativity>). Entre deux
réconciliations, l'état observé demeure celui résultant de la dernière
réconciliation effectuée. Le type et le nom d'une ressource sont immuables:
modifier l'un ou l'autre de ces champs équivaut à supprimer la ressource
existante et à en recréer une nouvelle.

=== Relations entre les ressources
Deux types de relations lient les ressources entre elles: la possession et le
dépendance. Une ressource peut avoir au plus un détenteur; celui-ci dispose des
droits de modification et de suppression sur la ressource, ainsi que le droit de
consulter la spécification et l'état observé de celle-ci. Le détenteur d'une
ressource donnée est simplement celui qui crée cette ressource. La dépendance
permet de consulter la spécification et l'état observé, sans pour autant pouvoir
modifier la ressource.

Le lien de dépendance à aussi un impact sur la planification: lorsqu'une
ressource $A$ dépend d'une ressource $B$, il est raisonnable de ne pas tenter de
réconcilier $A$ tant que $B$ n'est pas prête. En outre, ces relations ne sont
jamais transitives: si $A$ possède $B$, et $B$ possède $C$, $A$ ne possède pas
$C$. Le mécanisme exacte est décrit dans le #chapter-full-ref(
    <ch:system-design:scheduling>,
).

=== Catégories de ressources <ch:system-design:restypes>
Les ressources sont classées en deux catégories primaires: les ressources qui
peuvent être détenues, et celles ne pouvant pas l'être. Il s'agist
respectivement de ressources privées et de ressources mutualisée.

Une ressource privée dispose d'un détenteur unique, disposant seul des droits de
modification et de suppression sur celle-ci. Trois sous-catégories de ressources
privées existent, selon la nature de ce détenteur et la persistance de la
ressource à travers les redémarrages. Une ressource statique est détenue
conceptuellement par l'administrateur système et ne peut être modifiée qu'en
transmettant une nouvelle version de la configuration du système. Une ressource
dynamique est détenue par une autre ressource, qui décide seule de son contenu
et de son cycle de vie. Une ressource éphémère est détenue par une autre
ressource au même titre qu'une ressource dynamique, mais ne persiste jamais à
travers un redémarrage du système, indépendamment de la persistance de la
configuration elle-même; cette dernière catégorie est notamment utilisée pour
les ressources d'adresse ou de route obtenues dynamiquement par DHCP et qui,
selon la spécification du protocole, ne devraient pas persister entre deux
redémarrages~@bib-dhcp-rfc.

Une ressource partagée, à l'inverse, ne dispose d'aucun détenteur propre: elle
est détenue par l'orchestrateur, identifiée uniquement par son type et son nom.
Son fonctionnement précis, en particulier la motivation derrière son existence,
est détaillé dans le #chapter-full-ref(<ch:system-design:shared>). Il est à
noter qu'une ressource mutualisée peut, au même titre qu'une ressource privée,
être elle-même détentrice d'autres ressources: la distinction entre ces deux
catégories ne porte que sur l'impossibilité d'être détenue, non sur la capacité
à détenir.

Il convient de préciser que le qualificatif "privée" ne signifie pas que la
ressource est invisible au reste du système: sa spécification et son état
observé demeurent consultables par toute ressource déclarant une dépendance à
son égard, conformément au #chapter-full-ref(<ch:system-design:struct>). Seuls
les droits de modification et de suppression sont restreints à son détenteur.

Les ressources statiques sont les seules ressources que l'administrateur peut
modifier. En effet, le lien de possession n'étant pas transitif, lorsque
l'administrateur crée une ressource $A$ qui crée à son tour une ressource $B$,
le seul moyen pour l'administrateur d'agir sur la ressource $B$ est de passer
par $A$. Si l'utilisateur souhaite pouvoir interagir directement avec $B$, il
doit alors faire en sorte que celle-ci ne soit plus gérée par $A$, et ensuite de
la déclarer lui-même dans la configuration, ce qui rendrait cette ressource
statique. Cette distinction entre statique et dynamique permet de restreindre
l'accès de l'administrateur aux seules ressources pertinentes du point de vue de
la configuration, en masquant les détails d'implémentation qu'une ressource
statique délègue à ses enfants dynamiques. Cela rend aussi possible d'abstraire
une configuration complexe, nécessitant plusieurs ressources, derrière une seule
ressource simple, tel qu'illustré dans la #figure-num-ref(<cfgdyn>):

#include "../diagrams/cfgdyn.typ"

Il est ainsi commun dans les systèmes existants (netplan, systemd-networkd,
notamment) de configurer conjointement le lien réseau, les routes et les
adresses associées au sein d'une seule et même configuration. Dans le présent
système, un configuration de type `network:interface`~#bref(<cfgdyn-cfg>)
abstrait cette complexité en créant et en possédant plusieurs sous-ressources
dynamiques (`network:link`, `network:address` et `network:route`), chacune
correspondant à un aspect distinct~#bref(<cfgdyn-dyn>). La détenteur gère
entièrement la spécification de ces enfants: lorsque sa propre spécification est
mise à jour puis réconciliée, alors celui-ci mettra à jour la spécification de
ses enfants. L'administrateur du système n'a lui qu'a se préoccuper de la
spécification de la ressource parente. D'autre part, cela permet de modéliser
des aspect dynamique par nature, tel que la configuration du réseau via DHCP,
tel qu'illustré dans la #figure-num-ref(<cfgdhcp>):

#include "../diagrams/cfgdhcp.typ"

Une ressource de type `network:dhcp` est crée par l'utilisateur~#bref(
    <cfgdhcp-cfg>,
) et, en arrière plan, un client DHCP va configurer l'addresse et la route
réseau lorsque celles-ci deviendront disponible a travers le protocole #bref(
    <cfgdhcp-dyn>,
). Dans ce cas, si l'administrateur avait la capacité de modifier ces
sous-ressources, cela irait à l'encontre du fonctionnement nominal du DHCP.

=== Ressources mutualisée <ch:system-design:shared>
Le partage d'une ressource est donc naturellement réalisé par un lien de
dépendance. Toutefois, cela présuppose que la ressource soit crée par une autre
ressource ou par l'administrateur du système. En raison d'usages et d'habitudes
commune adoptée au sein de certains logiciels, certaines ressources sont
implicites à un point ou les déclarer manuellement serait contre productif. Mais
ne pas les déclarer explicitement soulève la problématique d'assigner la
responsabilité de créer et supprimer celle-ci. C'est notamment le cas des images
de conteneurs, comme illustré dans la #figure-num-ref(<cfgshared>):

#page(flipped: true)[
    #include "../diagrams/cfgshared.typ"

    Par exemple, deux configurations de conteneurs~#bref(<cfgshared-cfg>)
    indépendantes créent deux conteneurs qui semblent eux-aussi
    indépendants~#bref(<cfgshared-real>), mais qui pointent en réalité vers une
    même ressource, en l'espèce l'image~#bref(<cfgshared-conflict>). En raison
    des propriétés de la gestion des images dans un runtime de conteneurs, la
    création ne pose pas de problèmes; l'image sera téléchargée par le runtime
    puis utilisée par les conteneurs. En revanche, la suppression pose problème:
    un runtime ne supprime jamais les images de lui-même, et au fil de
    l'utilisation du système, les images prendraient de plus en plus de places
    sans jamais être supprimées.
]

Il n'est aussi pas possible de supprimer la ressource dès qu'un seul conteneur
est supprimée: l'image est toujours requise par l'autre configuration. Selon la
nature de la ressource sous-jacente, deux comportements seraient alors
possibles: soit la suppression échoue et la réconciliation reste bloquée, soit
la ressource est détruite et doit être recréée par l'autre configuration ce qui
est inefficace. La création d'une sous-ressource image par chaque conteneur est
également impossible, toute ressource ne pouvant être possédée que par un seul
parent.

C'est précisément ce problème que la catégorie des ressources mutualisées,
introduite au #chapter-full-ref(<ch:system-design:restypes>), permet de
résoudre. Contrairement à une ressource privée, une ressource mutualisée ne
possède aucune spécification propre: l'orchestrateur en est l'unique détenteur
et, en l'absence de tout paramètre, ne gère que sa création implicite et sa
suppression. Il suffit qu'une ressource déclare une dépendance vers une
ressource mutualisée pour que l'orchestrateur la crée si elle n'existe pas déjà;
si une autre ressource déclare ultérieurement dépendre de cette même ressource
mutualisée, aucune seconde instance n'est créée, la dépendance référençant
simplement l'instance existante.

L'orchestrateur décidant seul de la suppression d'une ressource mutualisée, la
règle appliquée est simple: dès qu'aucune ressource ne dépend plus d'elle, elle
est placée en cours de suppression, et suit alors les mêmes règles que toute
autre ressource, détaillées au #chapter-full-ref(<ch:system-design:deletion>).
De même, les principes de déclarativité présentés au #chapter-full-ref(
    <ch:system-design:declarativity>,
) s'appliquent également à cette catégorie: en particulier, même en l'absence de
spécification propre, une ressource mutualisée peut créer des sous-ressources
dont le contenu est arbitraire.

La #figure-num-ref(<cfgjoint>) illustre la résolution du conflit précédent grâce
à une ressource mutualisée:

#page(flipped: true)[
    #include "../diagrams/cfgjoint.typ"

    Dans la #figure-num-ref(<cfgjoint>), les deux conteneurs déclarent chacun
    une dépendance envers la même ressource mutualisée d'image~#bref(
        <cfgjoint-imgref>,
    ), plutôt que de créer chacun leur propre ressource. Cette ressource
    mutualisée unique~#bref(<cfgjoint-joint>) est ainsi seule responsable de la
    ressource physique correspondante~#bref(<cfgjoint-noconflict>). Il n'existe
    donc plus de conflit: si l'une des deux configurations change le nom de
    l'image référencée, cela crée simplement une nouvelle ressource mutualisée
    distincte, plutôt qu'un conflit sur l'ancienne.
]

Dans l'exemple des conteneurs, la réconciliation se déroule ainsi:
+ la ressource représentant la configuration du conteneur $A$ récupère d'abord,
    d'une manière ou d'une autre, le nom complet de l'image configurée, avec son
    hash;
+ à partir de cette information, elle déclare une dépendance envers une
    ressource mutualisée de type `container:image`, portant pour nom le nom
    complet de l'image avec son hash;
+ cette ressource mutualisée se charge alors de télécharger l'image sur le
    disque; cette opération, pouvant nécessiter plusieurs itérations, est
    effectuée en arrière-plan, la ressource conservant entre-temps le statut
    "pas prêt";
+ tant que la ressource `container:image` n'a pas atteint le statut "terminé",
    la ressource `container:instance` correspondante n'entreprend aucune autre
    action;
+ une fois l'image téléchargée, la ressource `container:image` passe au statut
    "terminé";
+ la ressource `container:instance` crée alors le conteneur;
+ le cycle de vie normal de la ressource se poursuit ensuite;
+ lorsqu'un conteneur dépendant de cette image est supprimé, le lien de
    dépendance correspondant est rompu;
+ si le nombre de dépendances entrantes de la ressource `container:image` tombe
    à zéro, l'orchestrateur ordonne sa suppression, ce qui efface l'image du
    disque;
+ si d'autres conteneurs dépendent encore de cette même image, aucune action
    supplémentaire n'est entreprise.

== Orchestrateur et réconciliation

=== Orchestrateur
L'orchestrateur est simplement le composant responsable de stocker l'ensemble
des ressources du système, de s'assurer que les réconciliations ont lieux dans
le bon ordre, et de garantir que le système se comporte correctement.

=== Contrôleur et déclarativité <ch:system-design:declarativity>
La réconciliation est le processus qui assure que l'état observé d'une ressource
converge vers sa spécification; le contrôleur est le composant logiciel qui en a
la responsabilité. Ce processus s'inscrit dans une boucle permettant de corriger
toute dérive: à chaque itération, le contrôleur observe l'état physique réel de
la ressource, le compare à sa spécification, puis produit les actions
correctives nécessaires. La #figure-num-ref(<decl>) illustre cette boucle:

#include "../diagrams/decl.typ"

Dans le cas de la configuration d'un lien réseau #footnote[
    Un lien réseau correspond à ce qui est communément appelé une interface
    réseau.
], par exemple, la spécification~#bref(<decl-cfg>) pourrait indiquer que le
statut administratif doit être "up" #footnote[
    Linux fait la distinction entre l'état administratif et l'état opérationnel
    d'une interface. L'état administratif indique si l'administrateur souhaite
    utiliser le périphérique pour le trafic. L'état opérationnel indique la
    capacité d'une interface à transmettre ces données
    utilisateur~@bib-linux-operstate.
]. L'état physique de l'interface (status administratif, status opérationnel,
adresse MAC, etc.) est d'abord récupéré~#bref(<decl-obs>) constituant ainsi
l'état observé, puis comparé à la spécification~#bref(<decl-diff>). Dans le cas
ou le status administratif serait "down", le système s'en rend compte et sait
qu'il doit exécuter l'équivalent de `ip link set up` afin de mettre en route
l'interface~#bref(<decl-actions>). Ce même mécanisme s'applique à toute
modification ou suppression de la ressource: une modification de la
configuration déclarative se traduit automatiquement par les actions correctives
adéquates.

En ce qui concerne les sous-ressources, lors de la réconciliation d'une
ressource donnée, celle-ci retourne d'une part le nouvel état observé, mais elle
déclarera aussi l'ensemble de ses enfants avec leur spécification. Le ressource
est libre de déterminer comme elle le souhaite le contenu de la spécification de
ces enfants spécification, la seule contrainte étant la validité vis-à-vis du
schéma du type de ressource. De fait, même en l'absence de tout paramètres,
comme dans une ressource mutualisée, rien m'empêche, lors de la réconciliation,
de créer des sous-ressources avec une spécification arbitraire. En outre,
supprimer un enfant de cette décaration reviens à demander la suppression de
celui-ci.

=== Orchestration de la réconciliation
Le système est composé de plusieurs ressources, chacune gérée par une logique de
réconciliation indépendante. Une réconciliation peut survenir de manière
périodique ou en réponse à un événement, tant interne, tel qu'une mise à jour de
la configuration, qu'externe, comme un conteneur qui s'arrête. À ce titre, deux
modèles d'orchestration sont envisageables: un modèle centralisé, dans lequel
une boucle unique appelle, à chaque itération, la méthode de réconciliation
d'une ressource donnée;. et un autre modèle modèle décentralisé, dans lequel
chaque contrôleur contient sa propre boucle, organisé de la manière qui lui
convient (une boucle par resource, une boucle par type, etc.). Le modèle
centralisé est illustré dans la #figure-num-ref(<ctrlloop>):

#include "../diagrams/ctrlloop.typ"

L'orchestrateur exécute une unique boucle: à chaque itération, il retire de la
file d'attente la ressource dont l'échéance de réconciliation est la plus
proche, délègue sa réconciliation au contrôleur correspondant, traite la réponse
de celui-ci (nouvel état observé, statut et sous-ressources déclarées), puis
replanifie la ressource selon le délai retourné. Dans ce modèle, le contrôleur
n'a pas connaissances des autre ressources, même celle sous son contrôle. La
réconciliation se repose uniquement sur les informations de la ressource
courante et d'éventuels détails sur d'autres ressources transmis par
l'orchestrateur sur la base des liens de possession ou de dépendance.

Le modèle décentralisé adopte l'approche opposée en déplaçant la boucle dans
chaque contrôleur, tel qu'illustré dans la #figure-num-ref(<indloop>):

#include "../diagrams/indloop.typ"

Chaque contrôleur exécute sa propre boucle de réconciliation, indépendante de
celle des autres contrôleurs, et détermine lui-même l'instant de la prochaine
réconciliation de chacune des ressources dont il a la charge, sans coordination
centrale.

Le modèle centralisé dispose d'une vue globale de toutes les ressources et de
leurs dépendances, ce qui permet de mieux coordonner la réconciliation à travers
plusieurs contrôleurs. Dans le cadre de ce modèle, l'orchestrateur pourrait
décider de ne pas réconcilier une ressource tant que ses dépendances ne sont pas
prêtes. À l'inverse, dans le modèle décentralisé, la ressource serait quand même
réconciliée quoi qu'il arrive; il incomberait alors au contrôleur de décider si
une dépendance non prête nécessite d'attendre ou non. Ce dernier modèle permet
en revanche davantage de flexibilité.

Cette vue globale donne également au modèle centralisé un avantage quant à la
réaction aux changements d'état et de spécification des ressources: un élément
central coordonnant l'ensemble, il devient trivial de propager ces changements
ou de redéclencher les réconciliations des ressources dépendantes. Un tel
mécanisme de propagation, dans un modèle décentralisé, nécessiterait un
mécanisme additionnel. La réaction aux événements externes (par exemple un
conteneur qui s'arrête, ou une interface réseau qui tombe) suit toutefois la
logique inverse: elle est nativement plus simple à traiter dans le modèle
décentralisé, tandis que le modèle centralisé nécessiterait l'ajout d'une
procédure dédiée permettant de notifier l'orchestrateur.

Le modèle centralisé présente enfin deux avantages mineurs. D'une part, il
permet d'assigner trivialement le parent d'une ressource: l'appel de
réconciliation étant effectué sur une ressource connue, et sa réponse contenant
les sous-ressources à créer, aucun risque de mauvaise assignation n'existe.
D'autre part, il permet de détecter aisément la panne d'un contrôleur, celui-ci
étant simplement considéré comme bloqué s'il ne répond pas dans le délai
imparti.

Deux critères supplémentaires ont été écartés de la comparaison. Le premier
concerne le nombre d'appels API: le modèle centralisé n'en requiert qu'un,
contre au moins deux pour le modèle décentralisé (l'un pour récupérer la
ressource, l'autre pour mettre à jour l'état); ce critère n'est pas retenu car
le nombre de ressources reste faible et les contrôleurs sont tous locaux. Le
second concerne la robustesse: le modèle décentralisé limite la panne à un
nombre restreint de ressources, mais ce critère n'est pas non plus retenu, les
contrôleurs étant de toute façon primordiaux au fonctionnement du système, si
bien que leur panne le rend inutilisable indépendamment du modèle choisi.

L'ensemble de ces points sont synthétisé dans le #table-num-ref(<scheduling>) :

#include "../diagrams/scheduling.typ"

Le modèle centralisé présente enfin une propriété plus fondamentale: la
réconciliation d'une ressource repose alors uniquement sur des éléments
entièrement déterminés par l'orchestrateur, à savoir sa spécification et son
état, ce qui garantit un comportement reproductible d'un cycle à l'autre. Un
contrôleur disposant de sa propre boucle pourrait, à l'inverse, introduire un
état ou une logique de planification propres, non visibles de l'extérieur, ce
qui nuirait à cette prévisibilité.

Au regard de l'ensemble de ces éléments, le modèle centralisé a été retenu comme
modèle d'orchestration. Sa propriété la plus important réside dans la pureté de
la réconciliation qu'il permet: celle-ci ne dépend alors que d'éléments
entièrement déterminés par l'orchestrateur, à savoir la spécification et l'état
de la ressource. Les avantages en termes de coordination des dépendances, de
réaction aux événements internes et de détection de pannes renforcent encore ce
choix, et l'emportent sur la flexibilité de planification et la capacité de
réaction aux événements externes offerte par le modèle décentralisé. Les
faiblesses du modèle centralisé (réaction aux événements externes spontanés,
flexibilité de la planification) sont compensées par un mécanisme explicite de
notification: chaque contrôleur peut signaler un événement externe à
l'orchestrateur, qui place alors la ressource concernée en tête de la file
d'attente. Les aspects de robustesse et de charge d'appels API sont considérés
comme secondaires dans un contexte où les contrôleurs sont locaux et en nombre
limité.

=== Boucle de réconciliation <ch:system-design:scheduling>
L'orchestrateur maintient une file d'attente de ressources à réconcilier. Lors
de l'insertion initiale d'une nouvelle ressource ou de la modification d'une
ressource existante, celle-ci est placée dans la file si ses dépendances sont
prêtes. Régulièrement, l'orchestrateur va prendre l'ensemble des ressources
arrivée à échéance et les réconcilier.

La réconciliation demeure purement séquentielle: il n'y a jamais plus d'une
ressource réconciliée en parallèle, ce qui exclut tout problème de concurrence
lors de ce processus. Afin de ne pas bloquer la boucle indéfiniment, une durée
maximale par réconciliation est imposée. En outre, lorsqu'une réconciliation
change le statut d'une ressource, en particulier son passage à un statut prêt ou
terminé, les ressources qui en dépendent sont à leur tour ajoutées à la file
d'attente comme décrit dans le #chapter-full-ref(<ch:system-design:events>).

Afin d'éviter toute incohérence, l'ensemble des ressources sont verrouillées le
temps d'une réconciliation. Toute modification de n'importe quelle ressource est
suspendue jusqu'à ce que la réconciliation en cours soit terminée ou interrompue
par expiration de la durée maximale impartie. Cela vaut aussi pour une mise à
jour de la configuration du système par l'administrateur.

=== Suppression des ressource <ch:system-design:deletion>
Lorsqu'une ressource est supprimée, l'ensemble des ressources qu'elle possède
est sont aussi supprimées. Afin de toujours garantir l'intégrité référentielle,
la suppression part des feuilles. En effet, une ressource ne peut pas être
complètement retirée du système tant qu'elle possède d'autres ressources ou des
dépendances entrantes. En outre, comme indiqué dans le #chapter-full-ref(
    <ch:system-design:shared>,
), les ressources mutualisées sont automatiquement placée en suppression dès
lors qu'elles n'ont plus aucune dépendance entrante. La suppression de cette
catégorie de ressource suit les même règles que les autres ressources.

=== Réaction aux événements <ch:system-design:events>
Un événement, interne ou externe, se traduit par la (re)planification d'une
ressource au sein de la file d'attente décrite dans le #chapter-full-ref(
    <ch:system-design:scheduling>,
). Un événement interne survient lorsque l'administrateur transmet une nouvelle
configuration: la ressource cible, une fois sa nouvelle spécification validée,
est immédiatement replanifiée. Un événement interne peut aussi survenir lorsque
le statut d'une ressource change après une réconciliation: les ressources qui en
dépendent sont alors planifiées afin de réagir à ce changement de statut. Un
événement externe survient lorsqu'un contrôleur constate, en dehors de tout
appel de réconciliation, un changement d'état pertinent pour une ressource dont
il a la charge, par exemple l'arrêt inattendu d'un conteneur; il lui suffit
alors de demander à l'orchestrateur de placer cette ressource dans la file. Dans
les deux cas, le mécanisme reste identique du point de vue de l'orchestrateur,
qui ne distingue pas l'origine de l'événement.

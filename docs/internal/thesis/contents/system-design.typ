#import "../lib.typ": *

= Conception du système

== Modèle de ressource

=== Structure d'une ressource <ch:system-design:struct>
Toute ressource partage une structure commune composée d'un type, d'un nom,
d'une spécification, d'un état observé, d'un statut et de métadonnées. Le type
fixe le schéma structurel de la spécification et de l'état observé, tandis que
le nom identifie de manière unique une instance au sein de ce type. La
spécification constitue l'état désiré, tandis que l'état observé reflète la
réalité constatée par le contrôleur; ce dernier, ainsi que les dépendances et
les sous-ressources associées, sont réévalués à chaque cycle de réconciliation,
mécanisme détaillé dans le #chapter-num-ref(<ch:system-design:declarativity>).
Le type et le nom d'une ressource sont immuables: modifier l'un ou l'autre de
ces champs équivaut à supprimer la ressource existante avant d'en recréer une
nouvelle. Entre deux réconciliations, l'état observé demeure celui résultant de
la dernière réconciliation effectuée.

=== Relations entre les ressources
// TODO: Un peu léger?
Deux types de relations liens les ressources entre elles: la possession et le
dépendance. Une ressource peut avoir au plus un détenteur; celui-ci dispose des
droits de modification et de suppression sur la ressource, ainsi que le droit de
consulter la spécification et l'état observé de celle-ci. La dépendance permet
de consulter la spécification et l'état observé, sans pour autant pouvoir
modifier la ressource.

Le lien de dépendance à aussi un impact sur le plan de la planification:
lorsqu'une ressource $A$ dépend d'ue ressource $B$, il est raisonnable de ne pas
tenter de réconcilier $A$ tant que $B$ n'est pas prête. En outre, ces relations
ne sont jamais transitive: si $A$ possède $B$, et $B$ possède $C$, $A$ ne
possède pas $C$.

=== Catégories de ressources <ch:system-design:restypes>
// TODO: Privé != que personne ne peut voir, ça veut juste dire que seul le détenteur peut modifier
// TODO: Une ressource mutualisée peut aussi détenir des ressources. C'est juste qu'elle même ne peut pas être détenue

Les ressources sont classées en deux catégories primaires: les ressources qui
peuvent être détenues, et celles ne pouvant pas l'être. Il s'agist
respectivement de ressources privées et de ressources mutualisée.

Une ressource privée dispose d'un détenteur unique, disposant seul des droits de
modification et de suppression sur celle-ci. Trois sous-catégories de ressources
privées existent, selon la nature de ce détenteur et la persistance de la
ressource à travers les redémarrages:
- une ressource statique est détenue conceptuellement par l'administrateur
    système et ne peut être modifiée qu'en transmettant une nouvelle version de
    la configuration du système;
- une ressource dynamique est détenue par une autre ressource, qui décide seule
    de son contenu et de son cycle de vie;
- une ressource éphémère est détenue par une autre ressource au même titre
    qu'une ressource dynamique, mais ne persiste jamais à travers un redémarrage
    du système, indépendamment de la persistance de la configuration elle-même.

Une ressource partagée, à l'inverse, ne dispose d'aucun détenteur propre: elle
est détenue par l'orchestrateur, identifiée uniquement par son type et son nom.
Son fonctionnement précis, en particulier la motivation derrière son existence,
est détaillé dans le #chapter-num-ref(<ch:system-design:shared>).

La distinction entre ressource dynamique et ressource statique permet deux
choses: d'une part elle rend possible le fait d'abstraire une configuration
complexe derrire une ressource plus simple, tel qu'illustré dans la
#figure-num-ref(<cfgdyn>):

#include "../diagrams/cfgdyn.typ"

Il est ainsi commun, dans les systèmes existants (netplan, systemd-networkd,
notamment), de configurer conjointement le lien réseau, les routes et les
adresses associées au sein d'une seule et même configuration. Dans le présent
système, un configuration de type `network:interface`~#bref(<cfgdyn-cfg>)
abstrait cette complexité en créant et en possédant plusieurs sous-ressources
dynamiques (`network:link`, `network:address` et `network:route`), chacune
correspondant à un aspect distinct~#bref(<cfgdyn-dyn>). La détenteur gère
entièrement la spécification de ces enfants: lorsque sa propre spécification est
mise à jour puis réconciliée, alors celui-ci mettra à jour la spécification de
ses enfants.

D'autre part, cela permet de modéliser des aspect dynamique par nature, tel que
la configuration du réseau via DHCP, tel qu'illustré dans la #figure-num-ref(
    <cfgdhcp>,
):

#include "../diagrams/cfgdhcp.typ"

Une ressource de type `network:dhcp` est crée par l'utilisateur #bref(
    <cfgdhcp-cfg>,
) et, en arrière plan, un client DHCP va configurer l'addresse et la route
réseau lorsque celles-ci deviendront disponible a travers le protocole #bref(
    <cfgdhcp-dyn>,
).

=== Ressources mutualisée <ch:system-design:shared>
Le partage d'une ressource privée est naturellement réalisé par un lien de
dépendance: plusieurs ressources peuvent consulter une même cible sans pouvoir
la modifier. Dans la plupart des cas, il est possible de déterminer un détenteur
unique pour une telle ressource partagée au sens large. Toutefois, il existe des
cas où une ressource équivalente existe de manière implicite au sein du système,
sans détenteur clair, comme illustré dans la #figure-num-ref(<cfgshared>).

#pagebreak(weak: true)
#page(flipped: true)[
    #include "../diagrams/cfgshared.typ"

    Par exemple, deux configurations de conteneurs~#bref(<cfgshared-cfg>), sans
    parent commun, créent deux conteneurs qui semblent indépendants~#bref(
        <cfgshared-real>,
    ), mais qui pointent en réalité vers une même ressource dynamique, en
    l'espèce l'image~#bref(<cfgshared-conflict>). Le système étant déclaratif,
    il faut disposer d'un moyen de supprimer cette image de manière déclarative,
    sans que sa suppression par l'une des deux configurations ne compromette
    l'autre.
]

Si l'utilisateur supprime l'une des deux configurations, la boucle de
réconciliation associée tenterait de détruire la ressource ainsi partagée, alors
que celle-ci reste requise par l'autre configuration. Selon la nature de la
ressource sous-jacente, deux comportements seraient alors possibles: soit la
suppression échoue et la réconciliation reste bloquée, soit la ressource est
détruite et doit être recréée par l'autre configuration.

C'est précisément ce problème que la catégorie des ressources partagées,
introduite à la section @ch:system-design:restypes, permet de résoudre.
Contrairement à une ressource privée, une ressource partagée ne possède aucune
spécification propre: l'orchestrateur en est l'unique détenteur et, en l'absence
de tout paramètre, ne gère que sa création implicite et sa suppression. Il
suffit qu'une ressource déclare une dépendance vers une ressource partagée pour
que l'orchestrateur la crée si elle n'existe pas déjà; si une autre ressource
déclare ultérieurement dépendre de cette même ressource partagée, aucune seconde
instance n'est créée, la dépendance référençant simplement l'instance existante.

L'orchestrateur décidant seul de la suppression d'une ressource partagée, la
règle appliquée est simple: dès qu'aucune ressource ne dépend plus d'elle, elle
est placée en cours de suppression, et suit alors les mêmes règles que toute
autre ressource, détaillées à la section @ch:system-design:deletion. De même,
les principes de déclarativité présentés à la section
@ch:system-design:declarativity s'appliquent également à cette catégorie: en
particulier, même en l'absence de spécification propre, une ressource partagée
peut créer des sous-ressources dont le contenu est arbitraire.

Le #figure-num-ref(<cfgjoint>) illustre la résolution du conflit précédent grâce
à une ressource partagée.

#pagebreak(weak: true)
#page(flipped: true)[
    #include "../diagrams/cfgjoint.typ"

    Dans le #figure-num-ref(<cfgjoint>), les deux conteneurs, à travers une
    référence d'image~#bref(<cfgjoint-imgref>), créent et gèrent finalement une
    même ressource logique~#bref(<cfgjoint-joint>), seule responsable de la
    ressource physique~#bref(<cfgjoint-noconflict>). Le problème n'est pas
    résolu en le supprimant, mais en le déplaçant: la ressource partagée~#bref(
        <cfgjoint-joint>,
    ) ne possédant aucune spécification, il ne peut exister aucun conflit à son
    sujet; si l'une des deux configurations change le nom de l'image référencée,
    cela crée simplement une nouvelle ressource partagée distincte, plutôt qu'un
    conflit sur l'ancienne.
]

Dans l'exemple des conteneurs, la réconciliation se déroule ainsi:
+ la ressource représentant la configuration du conteneur $A$ récupère d'abord,
    d'une manière ou d'une autre, le nom complet de l'image configurée, avec son
    hash;
+ à partir de cette information, elle déclare une dépendance envers une
    ressource partagée de type `container:image`, portant pour nom le nom
    complet de l'image avec son hash;
+ cette ressource partagée se charge alors de télécharger l'image sur le disque;
    cette opération, pouvant nécessiter plusieurs itérations, est effectuée en
    arrière-plan, la ressource conservant entre-temps le statut "pas prêt";
+ tant que la ressource `container:image` n'a pas atteint le statut "terminé",
    la ressource `container:instance` correspondante n'entreprend aucune autre
    action;
+ une fois l'image téléchargée, la ressource `container:image` passe au statut
    "terminé";
+ la ressource `container:instance` crée alors effectivement le conteneur;
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
converge vers sa spécification. À chaque cycle, le contrôleur observe l'état
physique réel, le compare avec l'état désiré et produit les actions correctives
nécessaires. La #figure-num-ref(<decl>) schématise cette boucle générique.

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

En ce qui concerne les sous-ressources, lors de la réconciliation d'une
ressource donnée, celle-ci retourna d'une part le nouvel état observé, mais elle
déclarera aussi l'ensemble de ses enfants avec leur spécification. Le ressource
est libre de déterminer comme elle le souhaite le contenu de la spécification de
ces enfants spécification, la seule contrainte étant la validité vis-à-vis du
schéma du type de ressource. De fait, même en l'absence de tout paramètres,
comme dans une ressource mutualisée, rien m'empêche, lors de la réconciliation,
de créer des sous-ressources avec une spécification arbitraire. En outre,
supprimé un enfant de cette décaration reviens a demander la suppression de
celui-ci, processus qui est expliqué dans le #chapter-full-ref(
    <ch:system-design:deletion>,
).

Le contrôleur est le composant responsable d'un ou plusieurs types de
ressources, pour lesquels il implémentera la logique déclarative: à partir de la
spécification et de l'état observé d'une ressource, il détermine et exécute les
actions correctives nécessaires pour faire converger l'état réel vers l'état
désiré, puis retourne le nouvel état observé, le statut résultant, et la liste
des sous-ressources dont il déclare l'existence ou la dépendance. Le contrôleur
ignore l'existence des autres ressources, même celles qu'il gère. Il maintient
uniquement un état minimal lui permettant d'effectuer des tâches de longue durée
(par exemple téléchargement). C'est uniquement lors de la réconciliation d'une
ressource spécifique que le contrôleur prend connaissance de la spécification et
des autres informations, de sorte à ce que la réconciliation d'une ressource
puisse se baser uniquement sur les informations fournie par l'orchestrateur.

=== Orchestration de la réconciliation
Deux architectures d'orchestration sont envisageables.

Le système est composé de plusieurs resources, chacune gérée par une logique de
réconciliation indépendantes. Une réconciliation peut arriver de manière
périodique ou en réponse à un événements, tant interne, tel qu'une mise a jour
de la configuration, qu'externe, comme un conteneur qui s'arrête. À ce titre,
deux modèles d'orchestration sont possible: un modèle centralisé, qui contient
une boucle unique, et au sein de cette boucle, fait appel a une méthode de
réconciliation sur chaque resource qui représente une itération. L'autre modèle
est le modèle décentralisé, dans lequel chaque contrôleur contient sa propre
boucle, organisé de la manière qui lui convient (une boucle par resource, une
boucle par type, etc.).

Le modèle centralisé est illustré comme suit:
#include "../diagrams/ctrlloop.typ"
#todo-inline[Commenter le diagram]

Tandis que le modèle décentralisé peut être illustré comme suit:
#include "../diagrams/indloop.typ"
#todo-inline[Commenter le diagram]

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
panne rend le système entier inutilisable.

#include "../diagrams/scheduling.typ"
#todo-inline[Commenter le tableau]

Au regard du #table-num-ref(<scheduling>), le modèle centralisé a été retenu
comme modèle d'orchestration. Les avantages en termes de coordination des
dépendances, de réaction aux événements internes et de détection de pannes
l'emportent sur la flexibilité de planification et la capacité de réaction aux
éventement externes offerte par le modèle décentralisé. Les faiblesses du
centralisé (réaction aux événements externes spontanés, flexibilité de la
planification) sont compensées par un mécanisme explicite de notification:
chaque contrôleur peut signaler un événement externe à l'orchestrateur, qui
place alors la ressource concernée en tête de la file d'attente. Les aspects de
robustesse et de charge d'appels API sont considérés comme secondaires dans un
contexte où les contrôleurs sont locaux et en nombre limité.

=== Boucle de réconciliation <ch:system-design:scheduling>
L'orchestrateur maintient une file d'attente de ressources à réconcilier. Lors
de l'insertion initiale d'une nouvelle ressource (= statut inconnu), ses
dépendances sont parcourues en largeur (BFS) afin de garantir que les racines de
l'arbre de dépendances soient réconciliées avant leurs feuilles. À chaque
réconciliation, le contrôleur retourne une durée minimale à attendre avant la
prochaine réconciliation de cette ressource, en l'absence d'événement.
L'orchestrateur impose en outre un délai minimal global entre deux
réconciliations pour éviter toute surcharge. Lorsque plusieurs ressources sont
éligibles à la réconciliation (leur délai d'attente est écoulé), la ressource la
plus en retard est traitée en premier. Une ressource peut également être
réveillée prématurément par un événement interne (mise à jour d'une dépendance)
ou externe (notification d'un contrôleur).

=== Suppression des ressource <ch:system-design:deletion>
Lorsqu'une ressource est supprimée, l'ensemble des ressources qu'elle possède
est sont aussi supprimées. Afin de toujours garantir l'intégrité référentielle,
la suppression part des feuilles (depth-first-search / DFS). En effet, une
ressource ne peut pas être complètement retirée du système tant qu'elle possède
d'autres ressources ou des dépendances entrantes. En outre, comme indiqué dans
le #chapter-full-ref(<ch:system-design:shared>), les ressources mutualisées sont
automatiquement placée en suppression dès lors qu'elles n'ont plus aucune
dépendance entrante. La suppression de cette catégorie de ressource suit les
même règles que les autres ressources.

=== Réaction aux événements internes
#todo[Réaction aux événements internes][
    - Comment on gère la réconciliation lorsque l'orchestrateur reçoit une
        modification?
    - => La ressource cible, après validation, est immédiatement ajoutée à la
        file de réconciliation
    - *Question*: il faudrait sans doutes permettre au contrôleur de retourner
        un délai minimal "fort" de sorte que
        $"schedule_time" = max("last" + "sys_min", "last" + "not_before_strong")$
]

La réconciliation est purement séquentielle. Il n'y a jamais plus d'une
ressource de réconcilié en parallèle et de fait, il ne peut pas y avoir de
problème de concurrence lors de la réconciliation. Afin de ne pas bloquer la
boucle indéfiniment, une durée maximale par réconciliation est imposée. En
outre, lorsque la réconciliation change d'état, en particulier lorsqu'une
ressource passe d'un status quelquonce à un status prêt ou terminé, les
ressources qui dépendent dessus seront ajouté à la file d'attente de
réconciliation.

=== Réaction aux événements externes
#todo[Réaction aux événements externes][
    - Comment on gère la réconciliation lorsque par exemple un conteneur exit de
        manière inattendue?
    - => Simplement permettre au contrôleur de demander à l'orchestrateur de
        placer une ressource (dont il est le contrôleur désigné) dans la file
]
Lorsque l'administrateur téléverse une nouvelle configuration, cela peut arriver
à n'importe quel moment. Afin d'éviter tout problème, une ressource en cours de
réconciliation est toujours verrouillées, ce qui mettrai en suspend la
modification de celle-ci jusqu'à ce que la réconciliation soit terminée. Pour
éviter les problèmes, une durée maximal de réconciliation est imposé.

#import "lib.typ": *
#import "../packages.typ": *
#import packages.drafting: *

#set page(
    numbering: "1/1",
    // margin: (left: 5cm, right: 5cm),
    // width: 210.0mm + 5cm,
    // height: 297.0mm,
)
#set heading(numbering: "1.")

= Conception du système

#todo[Conception du système][
    - transition
    - introduction du chapitre
    - plan interne
]

== Modèle de ressource
=== Structure d'une ressource
Toute instance de ressource partage une structure commune comprenant le type, le
nom, la spécification, l'état observé et diverses métadonnées #margin-note(
    side: left,
)[Compléter une fois que tout a été implémenté]. Le type détermine la structure
des champs de spécification et d'état observé, qui lui sont propres, tandis que
le contenu effectif de ces champs est défini indépendamment pour chaque
instance. Chaque instance est identifiée de manière unique par la combinaison de
son type et de son nom: le type fixe ainsi le schéma structurel, et le nom
désigne l'instance particulière portant les données conformes à ce schéma.
L'état observé est mis à jour par le contrôleur responsable de la ressource à
chaque réconciliation.

Les métadonnées d'une ressource continent le status de la ressource, la date de
création et de dernière modification, ainsi qu'une historique des changements de
status.

=== Status d'une ressource
Les status d'une ressource peuvent être décomposés en deux catégories: d'une
part les status contrôlés par le contrôleur, d'autre part les status contrôlés
par l'orchestrateur.

Une ressource dispose de plusieurs status en rapport avec la réconciliation: pas
prêt (_not ready_), prête (_ready_), en erreur (_error_), terminé (_done_), en
attente de suppression (_pending deletion_) en cours de suppression
(_deleting_), inconnu (_unknown_). Lors de la réconciliation, le contrôleur est
libre de retourne le status pas prêt, prêt, en erreur et terminé, les autres
status étant géré par l'orchestrateur.

// TODO: référencer le chapitre sur la suppression via @resource-deletion
Le status inconnu est assigné à la ressource lorsque celle-ci a été crée mais
qu'il n'y a pas encore eu de tentative de réconciliation. Il peut aussi être
assigné lorsqu'une ressource est en attente de suppression, puis que la
suppression est annulée et ce jusqu'à la prochaine réconciliation. Lors de la
réconciliation, le contrôleur retourne l'un des quatre status dont il a le
contrôle, et il décide à lui seul de la manière dont la ressource transitionne
entre ceux-ci. Lorsqu'une demande de suppression arrive, la ressource est placée
dans le status "en attente de suppression". Il n'est possible de sortir de ce
status que par deux moyens: soit la demande de suppression est annulée (en
demande la création de la ressource) et la ressource retourne dans un état
inconnu, soit la suppression n'est plus bloquée, auquel cas elle obtiendra le
status de "en cours de suppression". Depuis ce dernier status, aucun retour en
arrière n'est possible, la seule issue est la suppression de la ressource qui
l'efface complètement du système. À partir de ce moment, la ressource n'existe
plus et en recrée une du même type et du même nom recrée une toute nouvelle
ressource. Par ailleurs, lorsqu'une ressource est en cours de suppression, il
n'est plus possible d'en modifier sa spécification. La suppression pouvant être
longue, il est admis que l'état observé de la ressource soit mis à jour. Aussi,
une fois la ressource en cours de suppression, il n'est plus possible de bloquer
celle-ci; les liens de dépendance ajouté après-coup ne seront pas matérialisés.

// TODO: Introduire la machine d'état de transition (ou alors y déplacer avant?)
#include "diagrams/statustrans.typ"
// TODO: Commenter la machine d'état de transition

Enfin, certains status tel que le status inconnu ou le status d'erreur contient,
en plus du status, des détails sur la raison de celui-ci (message d'erreur, code
d'erreur, etc.). L'orchestrateur ne rend pas le status transitif (hormis la
suppression qui est transitive au sein des ressources possédées). Un contrôleur
peut toutefois décider de rendre les status qu'il contrôle transitifs. Le status
terminé signifie que la ressource ne sera plus réconciliée.

=== Relations entre les ressources <resource-links>
Les ressources disposent de deux types de relations réciproques et explicites
entre elles: un lien de possession, et un lien de dépendance. Implicitement il
existe un lien de création (puisqu'une ressource peut en crée une autre), ce
lien n'a pas d'utilité fonctionnelle et peut être déterminé sur la base des deux
autres liens. Le lien de possession est d'une part unique: une ressource ne peut
être possédée que par une seule autre entité, en revanche, une ressource peut
posséder plusieurs autre ressources. Le lien de possession permet de modifier,
voir supprimer la ressource cible. Il permet également de consulter le contenu
de la ressource cible, en particulier l'état observé de celle-ci. Enfin, le lien
de dépendance permet uniquement de consulter la ressource cible, sans pouvoir
effectuer d'autres actions. Par ailleurs, il est important de noter que tous ces
liens ont une incidence sur la gestion du cycle de vie d'une ressource: afin de
garantir l'intégrité des références, une ressource ne peut pas être supprimée
tant que des ressources qu'elle a crée ou qu'elle possède existe, et elle ne
peut pas être supprimée tant que d'autres ressources dépendent sur elle. Il est
important que pour pouvoir modifier une ressource possédée, il faut que le lien
soit direct. Un lien indirect (par exemple enfant d'enfant) ne permet pas la
modification. Les liens ne sont jamais transitifs.

#include "diagrams/cfgdyn.typ"

Enfin, le système empêche les dépendance cyclique. S'il détecte un tel cas,
l'ajout de la dépendance qui complèterait la boucle est empêché.
// TODO: ajouter le concept de weak-dependency (?)

=== Ressources partagées
Dans certains cas, il est souhaitable de pouvoir partager une ressource avec
d'autres ressources, sans pour autant leur donner un droit de modification et
c'est là qu'intervient le lien de dépendance. Dans le cas typique une ressource
partagée sera toujours crée par une seule autre entité, qui peut ensuite décider
de la partagée avec d'autres ressources. Par exemple, l'administrateur peut crée
un réseau de conteneur, puis deux conteneurs dépendant de ce réseau. Dans ce
cas, le réseau de conteneur et les deux conteneurs sont tous des racines, et
pour avoir accès au réseau, l'administrateur spécifie simplement son nom dans la
spécification des conteneurs. Cela en fait donc une ressource qui est crée et
partagée explicitement. Il existe toutefois certains cas ou une ressource est
crée de manière implicite en raison des convention d'usage, c'est notement le
cas des image de conteneurs tel qu'illustré dans la @cfgshared.

#pagebreak(weak: true)
#page(flipped: true)[
    #include "diagrams/cfgshared.typ"
    La @cfgshared montre que, dans le cas où l'utilisateur souhaite configurer
    deux conteneurs reposant sur l'image~`alpine:latest`~#bref(<cfgshared-cfg>).
    Chaque configuration génère un ensemble de sous-ressources disposant chacune
    de sa propre boucle de réconciliation~#bref(<cfgshared-dyn>) gérant une
    resource physique~#bref(
        <cfgshared-real>,
    ). Or, certaines de ces sous-ressources, bien qu'indépendantes du point de
    vue logique, gèrent en réalité la même ressource physique
    sous-jacente~#bref(
        <cfgshared-conflict>,
    ). Il en résulte un conflit potentiel car la resourcere serait géré par deux
    boucles différentes n'ayant pas conaissance l'une de l'autre.
]

En gros l'usage veut que les images soit implicite, or deux conteneurs qui
dépendent sur la même image bah qui a le droit de supprimer? Le "possesseur" n'a
pas connaissances des autre ressources, et même s'il en avait connaissance, ça
bloquerait la supresion du possessuer tant que n'importe quel autre conteneur
dépend dessus. La solution est d'introduire une sorte d'inversion des
dépendances via les ressources mutualisée: ces ressources ne disposent pas de
paramètres donc c'est safe de les crée, et puisqu'il n'y a pas de paramètres,
pas besoin de les modifier; on peut donc y transferer la gestion de la
suppression au niveau de l'orchestrateur et tout fonctionne. C'est illustré dans
la @cfgjoint.

Si l'utilisateur supprime l'une des deux configurations, la boucle de
réconciliation associée tentera de détruire la ressource partagée, alors que
celle-ci est encore requise par l'autre configuration. Selon la nature de la
ressource sous-jacente, deux comportements sont possibles: soit la suppression
échoue et la réconciliation reste bloquée, soit la ressource est détruite et
doit être recréée par l'autre configuration.

La solution est donc d'introduire la ressource mutualisée du: ce type de
ressource dispose de plusieurs parents pouvant la créer, mais son identifiant
constitue à lui seul sa spécification complète. Tout désaccord entre parents se
traduit donc par la création de ressources distinctes plutôt que par un conflit.
Dans l'exemple des conteneurs, la réconciliation se ferait donc comme suit:
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

#pagebreak(weak: true)
#page(flipped: true)[
    #include "diagrams/cfgjoint.typ"

    Dans la @cfgjoint, les deux conteneurs vont, à travers une `ImageRef`~#bref(
        <cfgjoint-imgref>,
    ), finalement créer et gérer une même resource logique~#bref(
        <cfgjoint-joint>,
    ), qui sera seule responsable de la resource physique~#bref(
        <cfgjoint-noconflict>,
    ). Le problème apparaît ici comme étant déplacé ailleur plutôt que résolu.
    La résolution concrète vient du fait que la resource mutualisé~#bref(
        <cfgjoint-joint>,
    ) ne possède pas de paramètres, et il ne peut donc pas y avoir de conflit;
    si une resource change le nom, cela va simplement crée une nouvelle
    resource.
]


=== Catégories de ressources <resources-types>
Les ressources sont séparée en trois catégories, qui regroupent les propritété
de ces différents liens: statique, dynamiques, et mutualisées tel qu'illustré
dans le @restypes.

#include "diagrams/restypes.typ"

Les ressources statiques sont crée et possédée par l'administrateur du système:
celui-ci peut modifier et supprimer les ressources à son bon vouloir. Elle ne
possède par ailleur pas de lien de dépendance avec leur créateur car
l'administrateur système n'est pas une entité réelle au sein du système. Ces
ressources ont aussi la particularité d'être des racines au sein d'un arbre
permettant de retracer l'origin de n'importe quel autre ressource.

Les ressources dynamiques sont simplement des ressources qui sont crée par une
autre ressource (peu importe la catégorie), et qui sont possédée par cette même
ressource.

Enfin, les ressources mutualisées ont la particularité de pouvoir être crée par
plusieurs ressources en même temps. Dans ce cas là, il est impossible de
déterminer une seule ressource là possédant, dès lors, elle conceptuellement
possédée par l'orchestrateur. En outre, la ou les ressources créant une
ressource mutualisée doivent aussi considérer cette ressource mutualisée comme
une dépendance. En effet, afin de savoir quand une ressource mutualisée peut
être supprimée, l'orchestrateur se base sur le nombre de dépendances; lorsque
celui-ci tombe à zéro, alors le processus de suppression, décrit
#todo-inline[référencer chapitre], est initié.

=== Synthèse

#include "diagrams/rels.typ"

== Réconciliation

=== Déclarativité
Le processus permettant de répercuter les modification apportée la spécification
d'une ressource sur l'objet physique sous-jacent s'appel la réconciliation. À
chaque réconciliation d'une ressource, le système va renouveller l'état actuel
de la ressource, le comparer avec la spécification, puis effectuer les actions
correctives pour faire converger l'état actuel avec la spécification tel
qu'illustré dans la @decl.

#include "diagrams/decl.typ"

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

=== Orchestration de la réconciliation
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

#include "diagrams/scheduling.typ"

Au regard du @scheduling, le modèle centralisé a été retenu comme modèle
d'orchestration. Les avantages en termes de coordination des dépendances, de
réaction aux événements internes et de détection de pannes l'emportent sur la
flexibilité de planification et la capacité de réaction aux éventement externes
offerte par le modèle décentralisé. À ce titre, l'orchestrateur inclura donc un
mécanisme permettant de aux contrôleur de remonter les événements externes en
vue de lancer une nouvelle réconciliation.

=== Boucle de réconciliation
Le système prend séquentiellement chaque ressource et applique cette logique de
réconciliation, une fois arrivé à la dernière ressource, il recommence, tel
qu'illustré dans la @ctrlloop.

#include "diagrams/ctrlloop.typ"

En intégrant la réconciliation à une boucle, cela permet de corriger les dérives
du système. Par exemple si un câble réseau est débranché puis rebranché, le
système s'en rendra compte a la prochaine réconciliation de la ressource
correspondante et effectuera les actions nécessaire.

La réconciliation est une file d'attente; lorsque des ressources sont ajoutées,
cela est fait de sorte que la racine d'un arbre de dépendance soit toujours la
première ressource, ensuite c'est du BFS. Cela concerne les ressources dans le
status inconnu. A chaque réconciliation, la ressource retourne le temps minimal
à attendre avant de tenter une nouvelle réconciliation en l'absence de tout
événements. Si rien ne vient initier une réconciliation, il est garanti qu'au
moins ce temps s'écoulera avant la prochaine réconciliation. Le système impose
aussi un temps d'attente minimale entre les réconciliations afin d'éviter toute
surcharge. En outre, lorsque la boucle choisi la prochaine ressource à
réconcilié, si plusieurs sont arrivée à expiration, celle étant expirée depuis
le plus longtemps sera faite en première.

=== Suppression d'une ressource <resource-deletion>
Au sein du chapitre @resource-links, il est mentionné que les références à
d'autres ressources doivent toujours être valide (que ce soit le créateur, le
détenteur, ou les dépendances). Lorsqu'une ressource est supprimée, alors toute
les ressource qu'elle détient sont supprimées elle aussi. En particulier, la
ressource parente restera en attente de suppression jusqu'à ce que toute les
ressources qu'elle détient soient elle-aussi supprimée; de fait, seul les
ressources "feuilles" dans l'arbre de détention peuvent être supprimées. À cela
s'ajoute le principe des dépendances: tant que d'autres ressources dépendantes
d'une ressource, alors celle-ci restera bloquée en attente de suppression, la
difference étant que la suppression n'est pas propagée au ressource qui
dépendent sur elle. Il est important de souligner que l'intégrité de la
référence du créateur est toujours maintenu car, dans le cas des ressources
dynamique, le créateur est aussi le détenteur, et dans les autres cas
(ressources statique ou mutualisée), le créateur est une entité "externe" (?) au
système (respectivement l'administrateur ou l'orchestrateur). Les ressources
mutualisée répondent donc aux même contraintes: elle ne peuvent pas être
supprimée tant que d'autre ressources dépendent sur elle, et leur suppression
n'est de toute manière initiée que lorsque plus aucune ressource ne dépende sur
elle. Une fois initiée, elle doit aussi satisfaire les règles de possession de
ressources, en effet, une ressource mutualisée peut aussi contenir des
ressources dynamiques vers qui la suppression sera propagée et devra être
complétée d'abord. Par ailleurs, dans le cas des dépendance, il faut que soit le
lien soit explicitement rompu, soit la ressource soit complètement et
entièrement supprimée; tant qu'elle est en suppression, cela compte comme un
blocage.


=== Réaction aux événements interne

=== Réaction aux événements externes

== Initialisation du système
#include "diagrams/sysinit.typ"

== Démarrage des processus
#include "diagrams/procstart.typ"

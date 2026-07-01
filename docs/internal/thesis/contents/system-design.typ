#import "../lib.typ": *
#import "../../packages.typ": *

= Conception du système

#todo[Conception du système][
    - Transition avec le chapitre précédant/amorce
    - Objectifs du chapitre
    - Plan interne
]

#todo-missing[Manques du chapitre conception du système tel que rédigé][
    - Génération du nom: le contrôleur se débrouille pour faire en sorte que les
        sous-ressources qu'il crée aient un nom unique
    - Parler d'idempotence
    - Parler de sécurité? Ou bien vu que c'est toujours des techno spécifique, à
        mettre dans l'implémentation?
]

#todo-question[Questions ouvertes][
    - Que se passe-t-il si par exemple on a une ressource `TlsPrivateKey` et
        qu'on dépend dessus? Peut-on lire la clef privée?
    - De manière générale, comment faire remonter l'erreur lorsque, durant la
        réconciliation, le contrôleur retourne une réponse qui n'est pas valide
        (par exemple des dépendances qui ne sont pas réalisables?)
]

== Modèle de ressource

=== Structure d'une ressource

Toute ressource partage une structure commune composée d'un type, d'un nom,
d'une spécification, d'un état observé, d'un status et de métadonnées. Le type
fixe le schéma structurel de la spécification et de l'état observé tandis que le
nom identifie de manière unique une instance au sein de ce type. La
spécification est l'état désiré, tandis que l'état observé reflète la réalité
constatée par le contrôleur. L'état observé ainsi que les dépendances et autre
sous-ressources sont établis à chaque réconciliation, tel qu'expliqué dans le
#chapter-full-ref(<ch:system-design:declarativity>). Par ailleurs, le type et le
nom d'une ressource sont immuable; modifier l'un ou l'autre de ces champs est
l'équivalent de supprimer la ressource avant les anciennes valeurs et en recrée
une nouvelle. Entre deux réconciliation, l'état observé est celui résultant de
la dernière réconciliation.

=== Statut d'une ressource

// TODO: Revoir cette section
Le statut d'une ressource se divise en deux catégories: les statuts contrôlés
par le contrôleur et ceux contrôlés par l'orchestrateur. Les statuts possibles
sont:
- inconnu (_unknown_): état initial avant la première réconciliation ou lors du
    démarrage
- pas prêt (_not ready_): le contrôleur indique que la ressource n'est pas
    encore opérationnelle.
- prêt (_ready_): la ressource est conforme à sa spécification et pleinement
    fonctionnelle.
- erreur (_error_): une anomalie empêche d'atteindre l'état désiré; des détails
    (code, message) peuvent accompagner ce statut.
- terminé (_done_): la ressource a atteint un état final et ne nécessite plus de
    réconciliation. Si sa spécification est modifiée, l'orchestrateur la replace
    automatiquement dans le status inconnu avant de lancer une nouvelle
    réconciliation.
- en cours de suppression (_deleting_): la suppression est démarrée et est
    irréversible.

Lors de la réconciliation, le contrôleur retourne obligatoirement l'un des
quatre statuts pas prêt, prêt, erreur ou terminé; les autres statuts sont gérés
exclusivement par l'orchestrateur. Dans le cas des ressources statiques, le
status "en cours de suppression" est déclenché par le retrait de la ressource de
la configuration. L'état d'erreur n'a pas de sémantique particulière lors de la
planification de la réconciliation.

La status en _cours de suppression_ est le seul status contrôlé par
l'orchestrateur à être transitif au sein des ressources détenues. En ce qui
concerne les status contrôlés par le contrôleur, ce dernier est libre de les
rendre transitif ou non, mais cela se fait manuellement lors de la
réconciliation. La machine d'états de transition des statuts est présentée dans
la #figure-num-ref(<statustrans>).
#include "../diagrams/statustrans.typ"

Tout ressource commence toujours dans le status inconnu~#bref(
    <statustrans-unk>,
), de là, lors de la réconciliation la ressource basculera dans le status
retourné par le contrôleur~#bref(<statustrans-to-err>). Cette transition ne peut
s'effectuer qu'une seule fois par ressource et par démarrage. En effet, depuis
ces status les ressources ne peuvent retourner dans le status inconnu que lors
du redémarrage. En outre, les ressources peuvent librement passer à l'un ou
l'autre des status déterminé par le contrôleur, sans règles particulières au
niveau de l'orchestrateur~#bref(<statustrans-to-rdy>). Si le contrôleur place la
ressource dans l'état terminé~#bref(<statustrans-to-done>), cette transition
n'est réversible que dans le cas ou la spécification de la ressource est mise à
jour. Enfin, depuis n'importe quel état, il est possible de se rendre vers
l'état en cours de suppression~#bref(<statustrans-to-del>), qui est
définitivement irréversible.


=== Relations entre les ressources

Deux types de relations liens les ressources entre elles: la possession et le
dépendance. Une ressource peut avoir au plus un détenteur; celui-ci dispose des
droits de modification et de suppression sur la ressource, ainsi que le droit de
consulter la spécification et l'état observé de celle-ci. La dépendance permet
de consulter la spécification et l'état observé, sans pour autant pouvoir
modifier la ressource. Le but de ce lien est de permettre de partager l'état
observé d'une ressource à une autre ressource ne se situant pas dans l'arbre de
possession. Dans le cas ou la ressource se trouve au sein de l'arbre de
possession, il est envisageable que les informations requises soit remontées au
premier parent commun puis retransmises. Toutefois, toute les ressources ne se
situent pas dans un seul et même arbre de dépendance, comme abordé au sein du
#chapter-full-ref(<ch:system-design:restypes>).

Ces liens ont divers impact en ce qui concerne la planification de la
réconciliation et la suppression des ressources, qui seront abordés
respectivement abordés dans les #chapters-full-ref(
    <ch:system-design:scheduling>,
    <ch:system-design:deletion>,
). Lors de l'ajout d'un tel lien, si l'une ou l'autre des ressources n'existe
pas #todo-inline[commenter remonter l'erreur?]. Similairement, l'ajout d'un lien
qui créerait un cycle est aussi empêché. En outre, ces relations ne sont jamais
transitive: si $A$ possède $B$, et $B$ possède $C$, $A$ ne possède pas $C$.
Outre cela, il n'y a pas de limites particulières sur le nombre de dépendance
entrante ou sortante.

=== Ressources partagées et ressources mutualisées <ch:system-design:shared>

Le partage d'une ressource est naturellement réalisé par un lien de dépendance:
plusieurs ressources peuvent consulter une même cible sans pouvoir la modifier.
Dans la pluspart des cas, il est possible de déterminer un seul et unique
détenteur d'une telle ressource partagée. Toutefois, il existe certains cas ou
la ressource partagée existe de manière implicite au sein du système, et à
travers différents arbres de possession, comme illustré dans la #figure-num-ref(
    <cfgshared>,
).

#pagebreak(weak: true)
#page(flipped: true)[
    #include "../diagrams/cfgshared.typ"

    Par exemple, deux configuration de conteneurs~#bref(<cfgshared-cfg>), sans
    parent communs, crée deux conteneurs qui semble être indépendant~#bref(
        <cfgshared-real>,
    ), mais ces deux conteneurs pointe en réalité vers une même ressource, en
    l'espèce, l'image~#bref(
        <cfgshared-conflict>,
    ). Le système étant déclaratif, il faut disposer d'un moyen de supprimer
    cette image de manière déclarative.
]

Si l'utilisateur supprime l'une des deux configurations, la boucle de
réconciliation associée tentera de détruire la ressource partagée, alors que
celle-ci est encore requise par l'autre configuration. Selon la nature de la
ressource sous-jacente, deux comportements sont possibles: soit la suppression
échoue et la réconciliation reste bloquée, soit la ressource est détruite et
doit être recréée par l'autre configuration.

Pour résoudre ce problème, le modèle introduit la notion de ressource
mutualisée. Une ressource mutualisée est identifiée de manière unique par son
type et son nom; elle ne possède aucun spécification. Contrairement aux
ressources dynamiques ordinaires, elle n'appartient à aucun possesseur unique:
l'orchestrateur en est le détenteur et, en raison de l'absence de paramètre, ne
gère que la suppression de la ressource. En raison de l'absence de
spécification, sa création est implicite: il suffit a une ressource de déclarer
une dépendance vers une autre ressource mutualisée et l'orchestrateur se
chargera de crée cette ressource. Si, ultérieurement, une autre ressource
déclare dépendre de cette même ressource mutualisées, il n'y a pas de deuxième
instance de crée, elle va simplement référencer l'instance existante.

L'orchestrateur décidant de la suppression d'une ressource mutualisée, la règle
est simple: dès lors qu'aucune autre ressource ne dépend sur une ressource
mutualisée, alors celle si est placée en cours de suppression, et va suivre le
même principe que les autres ressources, tel que détaillé au sein
#chapter-full-ref(<ch:system-design:deletion>). De même, les mêmes principes de
déclarativité, expliqués au sein du #chapter-full-ref(
    <ch:system-design:declarativity>,
) sont appliqués à cette ressource, en particulier, même en l'absence d'une
spécification propre, une ressource mutualisée peut crée des sous-ressources
avec des configuration arbitraires.


La #figure-num-ref(<cfgjoint>) illustre la résolution du conflit précédent grâce
à une ressource mutualisée.
#pagebreak(weak: true)
#page(flipped: true)[
    #include "../diagrams/cfgjoint.typ"

    Dans la #figure-num-ref(<cfgjoint>), les deux conteneurs vont, à travers une
    référence d'image~#bref(
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

Dans l'exemple des conteneurs, la réconciliation se ferait donc comme suit:
+ La ressource représentant la configuration du conteneurs $A$, d'abord
    récupérer d'une manière ou d'une autre le nom complet de l'image configuré,
    avec son hash.
+ Avec cette information, la ressource va déclarer une dépendance sur une
    ressource mutualisée de type `Image` ayant pour nom le nom complet de
    l'image.
+ Cette ressource mutualisée de type image va s'occuper de _pull_ l'image sur le
    disque. Cela sera fait en arrière plan car cela peut prendre plusieurs
    itérations. En attendant, la ressource aura le status pas prêt.
+ Tant que la ressource `Image` n'a pas le status terminé, la ressource
    `Container` ne fait rien d'autre.
+ Une fois l'image téléchargée, la ressource de type `Image` passe en status
    terminé.
+ Alors, la ressource de type `Container` va crée à proprement dit le conteneur.
+ _S'en suit le cycle de vie normal de la ressource._
+ Lorsque l'une ou l'autre des ressources `Contenainer` est supprimée, cela le
    lien de dépendance entre cette dernière et la ressource de type `Image` sera
    rompus.
+ Si le nombre de dépendance tombe à zéro sur la ressource de type `Image`,
    alors l'orchestrateur ordonne sa suppression, ce qui va effacer l'image du
    disque.
+ Dans le cas ou d'autres conteneurs dépenderaient sur `Image`, alors rien de
    plus ne se passerait.

=== Catégories de ressources <ch:system-design:restypes>

Les ressources sont classées en trois catégories selon leur rôle vis-à-vis des
liens de possession et de dépendance: les ressources statiques, les ressources
dynamiques, et les ressources mutualisées. Les ressources statiques sont créées
et possédées par l'administrateur système (entité externe au modèle), elles
constituent les racines de tout arbre de possession (l'administrateur système
n'étant pas représenté comme une entité concrète). Elles ne peuvent être
modifiées ou supprimées que par l'administrateur. Les ressources dynamiques sont
toutes les ressources qui sont crées et possédée par une autre ressource, quelle
que soit la catégorie de cette dernière. Le parent décide à lui seul quel est le
contenu d'une telle ressource, en se basant ou non sur sa propre spécification,
et quand en crée ou en supprimer une. Enfin les ressources mutualisées sont crée
de manière pseudo-implicite, lorsque une ressource $A$, peu importe sa
catégorie, déclare une dépendance sur une ressource mutualisée $B$.

Le #table-num-ref(<restypes>) résume les propriétés et les liens pour chaque
catégorie.
#include "../diagrams/restypes.typ"
Le #table-num-ref(<restypes>) synthétise les modes de création, de détention et
les liens de dépendance propres à chaque catégorie de ressource: les ressources
statiques, seules créées et détenues par l'administrateur, constituent les
racines des arbres de possession sans lien de dépendance une quelconque autre
ressource; les ressources dynamiques, créées par une autre ressource qui en
devient le détenteur, sont liées à leur parent par un lien de possession et non
par une dépendance; les ressources mutualisées, créées implicitement lorsqu'une
ou plusieurs ressources déclarent une dépendance vers un même type et un même
nom, sont détenues par l'orchestrateur et ne sont accessibles qu'à travers le
lien de dépendance. Les ressources mutualisées sont elles aussi considérée comme
des racine d'arbre, l'orchestrateur étant le système lui-même et non une entité
spécifique au sein de celui-ci. Cette classification garantit qu'à tout instant
chaque ressource possède un unique détenteur (administrateur, ressource parente
ou orchestrateur).

En outre, l'introduction de ressources dynamiques et statiques permet de
maintenir une stricte logique d'une ressource "virtuelle" = une ressource
"physique". Par exemple, il est commun dans les systèmes existant (netplan,
systemd-networking, ...) de configurer le lien réseau, les routes et les
addresses liée à ce lien au sein d'une seule et même "entité". Cette logique
peut être implémentée tel qu'illustré dans la #figure-num-ref(<cfgdyn>).

#include "../diagrams/cfgdyn.typ"
La #figure-full-ref(<cfgdyn>) illustre la manière dont une ressource parente de
type `Network`~#bref(<cfgdyn-cfg>) abstrait la complexité de la configuration
réseau en créant et en possédant plusieurs sous-ressources dynamiques (`Link`,
`Address` et `Route`) qui correspondent chacune à une entité physique distincte
(lien réseau, adresse IP, route)~#bref(<cfgdyn-dyn>), conformément au principe
"une ressource virtuelle = une ressource physique". Le parent détermine
librement la spécification de ces sous-ressources, en l'occurrence à partir de
sa propre spécification déclarative (ici, les champs up et address sont propagés
aux enfants avec les adaptations nécessaires).

=== Synthèse
#todo[Synthèse][
    - Introduire la synthèse
    - Introduire la figure synthétique
]

#include "../diagrams/rels.typ"

#todo-inline[Commenter le diagram de synthèse du modèle de ressource]

== Orchestrateur et réconciliation

=== Orchestrateur
L'orchestrateur est simplement le composant responsable de stocker l'ensemble
des ressources du système, de s'assurer que les réconciliations ont lieux, et de
garantir que le système se comporte correctement.

=== Contrôleur et déclarativité <ch:system-design:declarativity>

La réconciliation est le processus qui assure que l'état observé d'une ressource
converge vers sa spécification déclarative. À chaque cycle, le contrôleur lit
l'état physique réel, le compare avec l'état désiré et produit les actions
correctives nécessaires. La #figure-num-ref(<decl>) schématise cette boucle
générique.

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
est libre de déterminer comme elle le souhaite le contenu de cette
spécification, la seule contrainte étant la validité vis-à-vis du schéma du type
de ressource. De fait, même en l'absence de tout paramètres, comme dans une
ressource mutualisée, rien m'empêche, lors de la réconciliation, de créer des
sous-ressources avec une spécification arbitraire. En outre, supprimé un enfant
de cette décaration reviens a demander la suppression de celui-ci, processus qui
est expliqué dans le #chapter-full-ref(<ch:system-design:deletion>).

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
panne, mais en péril le système entier, donc tout s'arrête.

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

#todo[Suppression d'une ressource][
    - Que se passe-t-il quand une ressource est en cours de suppression et que
        une nouvelle dependence est ajoutée ou alors que la spec est modifiée?
        Rien: la dependence n'est pas "faite" et le système va remonter ça au
        contrôleur à la prochaine réconciliation.
]

Lorsqu'une ressource est supprimée, l'ensemble des ressources qu'elle possède
est sont aussi supprimées. Afin de toujours garantir l'intégrité référentielle,
la suppression part des feuilles (depth-first-search / DFS). En effet, une
ressource ne peut pas être complètement retirée du système tant qu'elle possède
d'autres ressources ou des dépendances entrantes. Dans le cas des dépendances
entrantes, la suppression effective sera bloquée jusqu'à ce qu'il n'y ait plus
aucune dépendance. Si une dépendance à une ressource en cours de suppression, ce
lien de dépendance ne sera pas sauvegarder par le système. De même, si la
spécification d'une ressource en cours de suppresion est modifiée, cette
modification sera elle aussi ignorée. Le système étant déclaratif, si cette
nouvelle version de la spécification persite à être déclarée par une autre
ressource, alors une fois la ressource entièrement supprimée, elle sera recrée.

En outre, comme indiqué dans le #chapter-full-ref(<ch:system-design:shared>),
les ressources mutualisées sont automatiquement placée en suppression dès lors
qu'elles n'ont plus aucune dépendance entrante. La suppression de cette
catégorie de ressource suit les même règles que les autres ressources.

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

== Initialisation du système
#todo[Initialisation du système][
    - Comment est-ce qu'un contrôleur est associé à une ressource? Durant la
        phase de démarrage, l'orchestrateur va faire un appel API particulier
        qui va retourner tous les types qu'un contrôleur sait gérer
]
#include "../diagrams/sysinit.typ"

== Démarrage des processus
#todo[Démarrage des processus]
#include "../diagrams/procstart.typ"

#import "../lib.typ": *

// TODO: parler des dépendances cycliques
#set page(numbering: "1/1")

= Conception du système

== Contrôleurs et Ressources
// === Contrôleur
// Les contrôleurs, abordé brièvement dans le chapitre précédent, encapsulent toute
// la logique permettant de configurer une partie du système suivant les
// spécification fournie par une ressource. Un contrôleur supporte plusieurs type
// de ressources, dont il définit lui-même le type, le schéma de la spécification,
// et une partie du schéma de l'état de la ressource. En outre, il implémente la
// logique permettant de vérifier qu'un champ se conforme aux schéma.

=== Modèle de ressource
Les ressources sont les données centrales du système; elle représente l'état
désiré ainsi que l'état actuel d'une entité au sein du système (par exemple une
interface réseau, un conteneur, etc.). Ces ressources sont composées de
plusieurs information principales: leur type, leur nom, leur spécification
(l'état désiré), leur status, l'état actuel persistant et l'état actuel
éphémère. Le type, le nom, et le status sont des champs standardisés au sein du
système, avec un schéma et des contraintes commune à toute les ressources. En
revanche, les autres informations (la spécification et les états) sont définit
par type de ressource. Outre permettre d'identifié quel schéma doit être
appliqué pour valider la spécification ou les états, le type peut être combiné
avec le nom afin d'identifier de manière unique et stable une ressource au sein
du système. S'agissant d'un identifiant stable, il n'est pas possible de
modifier le nom ou le type d'une ressource, cela reviendrait a totalement
supprimer celle portant l'ancien nom, et en créer une nouvelle.

Le status


La spécification et l'état actuel doivent corresponde à un schéma, et chaque
type de ressource a son propre schéma pour chacune de ces deux informations.

Les ressources représentent tout d'abord un modèle de configuration, avec des
schémas de données et une logique de validation. Ce schéma peut être concrétisé
en fournissant un ensemble de valeur à ce schéma, et en nommant cet ensemble. La
ressource devient alors une entité au sein du système


La ressource est la donnée centrale du système et représente deux choses: d'une
part un ensemble de schémas et les valeurs acceptables de ceux-ci, et d'autre
part, la logique permettant, sur la base d'une spécification conforme aux
schémas, de configurer le système de manière adéquate.

Le système est organisé autour de ressources. Pour rappel une ressource contient
un type, un nom, une spécification et un état. De surcroit, chaque type de
ressource est géré par un contrôleur. Ce concept, brièvement introduit dans le
chapitre précédent, est le processus qui implémente toute la logique nécessaire
afin de configurer le système en respectant la spécification d'une ressource. Le
fonctionnement exacte d'un contrôleur sera détaillé dans #todo-inline[réf. ch.].




Bien que la majorité du contenu de l'état soit défini indépendamment entre
chaque ressource, il existe un contenu commun: #todo-inline[À faire].

Les ressources du système ont naturellement des liens entre elles. Il existe en
l'occurrence trois liens:
+ le lien de création
+ le lien de détention
+ le lien de dépendance

Le lien de création, à lui seul, n'a pas de sémantique particulière au sein du
système, il permet plutôt de tracer l'origine du ressource à travers l'arbre de
création. Toutefois, dans la majorité des cas, le lien de création implique un
lien de détention dans le même sens. Au sein du système, seul le détenteur peut
modifier une ressource. En outre, le lien de détention permet au détenteur de
recevoir, avec son propre état, l'état des ressources qu'il détient. Il est
aussi souhaitable de pouvoir consulter l'état de ressources qui ne sont pas
directement détenu, afin d'effectuer cela, le lien de dépendance est introduit.
Celui-ci a deux propriété, d'une part, il donne accès a celui qui dépend d'une
cible, à l'état de ses cibles lors d'une réconciliation, et en second lieu, tant
qu'une ressource est une dépendance d'autres ressources, celle-ci ne peut pas
être supprimée.

Sur cette base, il est possible d'identifier trois types de ressources qui se
distingue par leur liens:
+ les ressources statique
+ les ressources dynamique
+ les ressources mutualisée

#include "../diagrams/restypes.typ"

Le @restypev présente les différentes ressources avec leur liens. Dans le cadre
des ressources statiques, celles-ci sont détenu par l'administrateur, et
naturellement, seul celui-ci peut les modifier ou les supprimer. Une entité
n'étant pas une ressource, il n'existe pas de lien de dépendance (il serait
illogique de posséder un "état de l'administrateur"). Les ressources dynamiques
sont aussi un lien naturel: le créateur détient la ressource qu'il a crée, et
cette ressource dépend de son créateur. De cette manière, une ressource ne peut
pas devenir orpheline, et il est toujours possible de tracer son existence a une
ressource qui existe elle-aussi. Enfin les ressources mutualisée présentes des
particularité: d'une part, une seule et même ressource mutualisée peut être crée
par plusieurs ressources. Afin d'éviter les conflits de modification, tout champ
modifiable est simplement absent de ces ressources (elles n'ont donc qu'un type
et un nom; la spécification étant vide). Dans ce cas spécial, l'orchestrateur
est le détenteur de la ressource, cela est nécessaire pour la suppression. En
effet, celui-ci va, à chaque itération, tenter de supprimer la ressource, or,
compte tenu du fait que lorsqu'une ressource mutualisée est crée, la ressource
l'ayant crée dépend automatiquement de cette ressource mutualisée, la
suppression n'est possible que lorsque cette ressource créatrice est supprimée.

Ce mécanisme est introduit pour faire face à une problématique bien précise:
représenter une ressource qui est partagée de manière implicite. Une ressource
est considérée comme partagée dès lors que d'autres ressources dépendent sur
elle. Par exemple, un réseau de conteneur est une dépendance des conteneurs s'y
trouvant, et dans ce cas, il est naturel que l'administrateur définisse
explicitement une telle ressource. Toutefois, il existe deux cas ou une
ressource est partagée, mais est crée implicitement:
+ les dossiers au sein d'un système de fichier
+ les images de conteneurs

#set page(flipped: true)
#include "../diagrams/cfgshared.typ"
#set page(flipped: false)

#set page(flipped: true)
#include "../diagrams/cfgjoint.typ"
#set page(flipped: false)


En résumé, les dépendances (graph illustra les 3 cas)
#include "../diagrams/rels.typ"

=== Contrôleurs de ressources
- Réseau
- Conteneurisation (interaction avec le runtime, injection de secrets)
- Stockage et volumes

=== Orchestration de la réconciliation
- Boucle de contrôle (séquence, parallélisme/backoff, tâches longues)
- Ordonnancement et dépendances
- Gestion des erreurs et retry
- Suppression d'une ressource (ordre inverse, nettoyage)

=== Résolution des ressources concrètes
- Association interface physique / disque via attributs (MAC, nom, etc.)

=== Réaction aux événements
- Événements internes (changement d'état d'une sous-ressource)
- Événements externes (déclencheurs matériels, signaux)

== Composants
=== Vue d'ensemble des composants
Il existe cinq composants principaux dans le système:
+ l'init
+ le superviseur (_supervisor_)
+ le contrôleur principal (_core controller_)
+ l'api
+ les différents contrôleurs

L'init, qui est inclut dans le _initrd_ est simplement chargé de trouver le
disque système racine et de le monter puis passe la main au superviseur
#todo-inline[Expliquer pourquoi ça existe].

Le supervisor est chargé de mettre en place l'environement initial du système
(systèmes de fichier, arborescence, etc) et le chargement de la configuration de
démarrage #todo-inline[Référencer l'explication des différentes config]. En
outre il s'occupe du _process reaping_ et de la gestion des pannes du système
#todo-inline[Expliquer ce que c'est le _process reaping_]. Une fois tout cela
fait, il démarre le contrôleur principal (_core controller_).

Le contrôleur principal à plusieur tâches: d'une part il s'occupe de charger et
déchiffrer la configuration ainsi que l'état. Une fois cela fait, il sait
quelles fonctionalités sont disponible sur le système et démarrage tout d'abord
le contrôleur réseau puis l'API (pour autant que tout cela soit activé). Une
fois cela fait, il démarre les autres contrôleur et attends que tous soient
prêt. Une fois qu'ils sont tous prêt, la réconciliation peut démarrer.

Dans la réconciliation, chaque contrôleur va s'adonner à ses tâches, par exemple
monter les volumes persistents, monter les interfaces réseau, etc.

// Diagram de qui lance quoi
#include "../diagrams/procstart.typ"

// Diagram de la séquence de démarrage (qui est un peu expliquée au dessus)
#include "../diagrams/sysinit.typ"

=== Communication entre les composants
Les composants sont strictement isolés. En particulier, le superviseur et l'API
ne communiquent qu'avec le gestionnaire d'état. De même, chaque contrôleur ne
communique qu'avec le gestionnaire d'état et les processus qu'ils auraient
éventuellement lancés.


#todo[Schéma de qui communique avec quoi]

== Configuration
=== Sources de configuration
- disque, cloud-init, réseau, priorité

=== Validation de la configuration
- schéma, dépendances cycliques

== Administration
=== API et authentification
- Protocole (TLS, mTLS, token)
- Amorçage de l'authentification sur système vierge (cloud-init, clé USB)

=== Gestion des secrets
- stockage, chiffrement, injection dans les conteneurs

=== Observabilité
- Collecte et exposition des métriques (hôte et conteneurs)
- Agrégation et export des logs

== Cycle de vie du système
=== Immutabilité du système de base
=== Installation initiale
- partitionnement automatique, push de configuration

=== Mise à jour A/B et retour arrière
=== Mode maintenance
- shell restreint, accès réseau, bascule
=== Personnalisation de l'image
=== Méthodes de démarrage
=== Sauvegarde et restauration

== Persistence
=== Partitionnement des disques
- système, cache, données

=== Chiffrement des partitions
- TPM, FIDO2, passphrase, déverrouillage automatique/manuel

== Sécurité
=== Isolation
- conteneurs, composants internes, moindre privilège

=== Surface d'attaque et réduction
=== Menaces et contre-mesures

/*
= Conception du système

== Composants
=== Vue d'ensemble des composants
=== Communication entre les composants
=== Cycle de vie des composants

== Ressources
=== Modèle de ressource
=== Contrôleurs de ressources
=== Orchestration de la réconciliation
=== Boucle de contrôle
=== Suppression d'une ressource
=== Réaction à un événement interne
=== Réaction à un événement externe
=== Persistence de l'état
=== Résolution des ressources concrètes

== Configuration
=== Sources de configuration
=== Validation des ressources

== Administration
=== Authentication et permissions
=== Authentication sur un système non-initialisé
=== Collecte et exposition des métriques et des logs
=== Streaming d'information via l'API

== Cycle de vie du système
=== Immutabilité
=== Installation initiale
=== Mises à jour A/B
=== Mode maintenance

== Persistence
=== Partitionnement des disques
=== Chiffrement des données

== Sécurité
=== Isolation
=== Surface d'attaque
=== Menaces et contre-mesures
*/

/*
= Conception du système

== Composants
=== Communication entre les composants
=== Cycle de vie des composants

== Ressources
=== Modèle de ressource
=== Orchestration de la réconciliation
=== Suppression d'une ressource
=== Boucle de contrôle
=== Réaction à un événement interne
=== Réaction à un événement externe
=== Persistence de l'état
=== Résolution des ressources concrètes
=== Contrôleurs de ressources

== Gestion de la configuration
=== Sources de configuration
=== Validation des ressources

== Administration
=== Authentication et permissions
=== Authentication sur un système non-initialisé
=== Collecte et exposition des métriques et des logs
=== Streaming d'information via l'API

== Cycle de vie du système
=== Immutabilité
=== Mises à jour A/B
=== Installation initiale
=== Mode maintenance

== Persistence
=== Chiffrement et des données
=== Partitionnement des disques

== Sécurité
=== Isolation
=== Surface d'attaque
=== Menaces
*/


/*
== Architecture générale
#todo[Architecture générale][
    Le schéma C4 mais en mieux
]

== Démarrage du système
#todo[Démarrage du système][
    Schéma du processus de boot (init -> supervisor -> les contrôleurs)
]

== Orchestration de la réconciliation

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

<diagram>

Au regard du @scheduling, le modèle centralisé a été retenu comme modèle
d'orchestration. Les avantages en termes de coordination des dépendances, de
réaction aux événements internes et de détection de pannes l'emportent sur la
flexibilité de planification et la capacité de réaction aux éventement externes
offerte par le modèle décentralisé. À ce titre, l'orchestrateur inclura donc un
mécanisme permettant de aux contrôleur de remonter les événements externes en
vue de lancer une nouvelle réconciliation.

== Modèle de ressources

=== Structure d'une resource
Chaque resource est composée d'un identifiant unique, d'une spécification (=
l'état désiré), et d'un état (= l'état actuel). En outre, une resource peut
avoir, des enfants, ou des dépendances. L'ensemble de ces données est stockées
et permet de constituer une resource. Les relations inverses (parent et
dépendants) sont calculés dynamiquement.

La mise à jour de la spécification est décrite dans le @sub-restype. En ce qui
concerne la mise à jour de l'état, celui-ci est géré par un contrôleur qui
implémente la logique de réconciliation.

Il est nécessaire de séparer le lien de parenté du lien de dépendance: le lien
de parenté permet de créer, modifier, ou supprimer une ressource enfant, tandis
que le lien de dépendance permet d'empêcher la ressource sur cible d'être
supprimée tant que d'autres ressources dépendent sur elle. Dans la pluspart des
cas, lorsqu'un parent crée un enfant, l'enfant est considéré automatiquement
comme dépendant du parent.

=== Liens entre les ressources
Trois liens entre les ressources, et bien entendu les liens réciproques:
- le lien de création: c'est simplement la ou les ressources qui ont donné la
    spécification initiale.
- le lien de détention: ce lien permet au détenteur de modifier et supprimer la
    ressource.
- le lien de dépendance: lorsqu'une ressource dépend sur une autre ressource, la
    ressource cible (celle sur laquelle la ressource dépend) ne peut pas être
    supprimée tant qu'il existe au moins une ressource qui dépend sur elle.
    Outre cela, le lien de dépendance permet d'accèder à l'état de la ressource
    cible lors de la réconciliation de la ressource source.

=== Types de resources <sub-restype>
Sur la base de ces liens, il est possible de séparer les ressources en trois
catégories: statique, dynamique et mutualisée.

<diagram>

=== Dépendances et hiérarchie
Une resource peut dépendre d'autres resources. Ce lien de dépendance implique
trois choses: d'une part, même si elle n'est pas propriétaire de la resource
peut acceder à l'état à la spécification de sa dépendance, sans pouvoir la
modifier. D'autre part, il sera impossible de supprimer complètement les
resources sur laquelle dépend la resource tant que cette dépendance existe.
Enfin, la resource sur laquelle il y a dépendance pourra acceder à la
spécification et a l'état de ces resources, toujours sans pouvoir les modifier.

Le liens de parenté est une version plus forte du lien de dépendance; un parent
est celui ayant crée la resource et peut donc la modifier et la supprimer. Ses
enfants sont aussi considérés comes des dépendances et donc bloquent la
suppression complète jusqu'à ce qu'ils aient été supprimés.

Enfin il reste le cas particulier des resources mutualisés; celles-ci possèdent
plusieurs parent, mais ceux-ci ne disposent que d'un pouvoir de création et de
modification, et non le pouvoir de suppression. À l'inverse, une resource
mutualisée ne peut pas être supprimées tant que ses parents en ont besoins.
Compte tenu de cela, les resources mutualisés sont aussi considérée comme des
racines, sur lesquelles dependents les "parents".

=== Suppression des resources
Lorsqu'une demande de suppression d'une ressource est enregistrée, le système va
marquer la resource cible comment devant être supprimée. Tous les enfants de la
resource, qui sont aussi des dépendances, sont récursivement marqués pour
suppression. Tant qu'une resource possède encore des dépendances, celle-ci ne
peut pas être complètement supprimées.

=== Resources mutualisée
Les resources mutualisé sont des resources qui sont partagées (donc sur
lesquelles d'aures resources dependents), et qui sont crée de manière implicite
par ces resources dépendantes. Dans la majorité des cas, une resource implicite
peut être supprimée au profit d'une resource statique ou dynamique sans péjorer
la facilité d'utilisation. Il existe certains cas ou cela n'est toutefois pas
possible, notament le cas pour les images de conteneurs. Bien que
"physiquement", le conteneur et l'image soit deux resources séparées mais liées,
la majorité des systèmes de conteneurisation permettent de gérer l'image
directement avec le conteneurs et opèrent en arrière plan pour dé-dupliquer le
tout. Ils offrent aussi la possibilité de gérer manuellement les images. En
contrepartie, il est nécessaire de disposer d'un mécanisme de "garbage
collection" qui supprimera les images non-utilisée, même lorsque celles-ci ont
été sciement télécharger. Il faut donc introduire le concept de resource
mutualisé.

Afin d'illustrer la problématique de ces resources, l'exemple des images de
conteneurs.

<diagram>

La @cfgshared montre que, dans le cas où l'utilisateur souhaite configurer deux
conteneurs reposant sur l'image~`alpine:latest`~#bref(<cfgshared-cfg>). Chaque
configuration génère un ensemble de sous-ressources disposant chacune de sa
propre boucle de réconciliation~#bref(<cfgshared-dyn>) gérant une resource
physique~#bref(
    <cfgshared-real>,
). Or, certaines de ces sous-ressources, bien qu'indépendantes du point de vue
logique, gèrent en réalité la même ressource physique sous-jacente~#bref(
    <cfgshared-conflict>,
). Il en résulte un conflit potentiel car la resourcere serait géré par deux
boucles différentes n'ayant pas conaissance l'une de l'autre.

#set page(flipped: false)

Si l'utilisateur supprime l'une des deux configurations, la boucle de
réconciliation associée tentera de détruire la ressource partagée, alors que
celle-ci est encore requise par l'autre configuration. Selon la nature de la
ressource sous-jacente, deux comportements sont possibles: soit la suppression
échoue et la réconciliation reste bloquée, soit la ressource est détruite et
doit être recréée par l'autre configuration.

La solution est donc d'introduire la ressource mutualisée du @sub-restype: ce
type de ressource dispose de plusieurs parents pouvant la créer, mais son
identifiant constitue à lui seul sa spécification complète. Tout désaccord entre
parents se traduit donc par la création de ressources distinctes plutôt que par
un conflit. Dans l'exemple des conteneurs, la réconciliation se ferait donc
comme suit:
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

<diagram>

Dans la @cfgjoint, les deux conteneurs vont, à travers une `ImageRef`~#bref(
    <cfgjoint-imgref>,
), finalement créer et gérer une même resource logique~#bref(<cfgjoint-joint>),
qui sera seule responsable de la resource physique~#bref(<cfgjoint-noconflict>).
Le problème apparaît ici comme étant déplacé ailleur plutôt que résolu. La
résolution concrète vient du fait que la resource mutualisé~#bref(
    <cfgjoint-joint>,
) ne possède pas de paramètres, et il ne peut donc pas y avoir de conflit; si
une resource change le nom, cela va simplement crée une nouvelle resource.

#set page(flipped: false)

=== Synthèse du model de resource

// En somme, deux liens existent entre les resources: le lien de parenté, qui
// permet de modifier et lire une resource, et le lien de dépendance, qui permet de
// lire une resource sans pouvoir y modifier. En outre, le lien de dépendance a un
// effet bloquant, empêchant la resource cible d'être supprimée tant qu'une autre
// resource dépend sur elle.

// En somme, les resources sont décomposée en 3 catégories: statiques, dynamiques,
// et mutualisées. Deux liens existent entre les resources: le lien de parenté, et le lien de dépendance: le

Ces catégories régissent qui peut executer quelles actions sur ces dernières, en
particulier une resource statique ne peut être crée et modifiée que par
l'administrateur système, une resource dynamique peut être crée par n'importe
quelle autre resource et modifiée uniquement par celle qui l'a crée, et enfin

<diagram>
*/

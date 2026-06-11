= Réconciliation d'une resource

Chaque type de resource dispose de sa propre logique de réconciliation,
toutefois, elles suivent toutes une procédure commune. La commande qui initie la
réconciliation contient l'identité de la resource à réconcilier, sont état
désiré (spécification), ainsi que l'état actuel calculé durant l'appel
précédent.

Tout d'abord l'état actuel va être recalculé via `refresh()`. Ensuite, `plan()`
va comparer le nouvel état actuel avec l'état désiré et sortir un plan qui
représente les actions a entreprendre _immédiatement_ pour faire converger les
deux. Il est important de noter ici que _immédiatement_ veut dire que l'ensemble
des actions doit être réalisable en un temps court. Si l'action est de longue
durée (par exemple un téléchargement), alors celle-ci constitue la seul action
du plan. Une fois le plan calculé, celui-ci peut être appliqué. Lorsque l'action
entreprise prende longtemps, alors celle-ci est lancée en arrière-plan, de sorte
à ce que la fonction `apply()` retourne dans un délai assez bref. Cette
exécution en arrière plan doit être fait de sorte a ce que un appel ultérieur a
`reconcile()` ne génère pas de nouvelles tâches d'arrière-plan si cela n'est pas
nécessaire (pour ne pas télécahrger un fichier deux fois par exemple).

Enfin, la fonction `update()` va prendre l'ensemble de ces données, et produire
une réponse à transmettre à l'appelant. Cette fonction est utile car, dans le
cadre d'un apply, c'est ici que l'état actuel peut être recalculé une seconde
fois, toutefois, si le plan ne possède aucun changement, alors on peut
réutiliser l'état que nous avions déjà récupérer dans `refresh()`.

= Orchestration de la réconciliation

Chaque resource dispose de sa propre logique de réconciliation, gérée au sein
d'un "manager". Afin d'avoir un système résilient, il est nécessaire d'apeller
cette logique de réconciliation de manière régulière. Pour ce faire, deux
approches fondamentalement opposées existent: une approche centralisée et une
approche décentralisée.

Par approche centralisée, il faut comprendre qu'un seul composant est
responsable de parcourir la liste des resources et de transmettre des demandes
de réconciliation au sous-système responsable. À l'inverse, dans l'approche
décentralisée, le sous-sytème responsable de la resource va lui-même planifier
sa propre boucle.





= Aaaaaa

La gestion du système de manière déclarative se fait au travers d'un fichier de
configuration. Celui-ci décrit l'ensemble des aspects du système qui doivent
être configurés. Cela inclu la configuration d'une interface réseau, de routes
IP, de conteneurs, ou bien encore de disques.

Afin de faciliter la gestion de ces aspects, ceux-ci sont séparés en
"resources". Pour chaque resource, sa réconciliation passe par 4 étapes:
- le `refresh`, ou on va récupérer l'état actuel de la resource
- le `plan`, ou

== Composants

La solution est découpée en plusieurs composants. Chaque composant est
responsable d'un aspect particulier de la gestion du système. Un certains nombre
de composant peuvent être regroupés sous "managers". Ces "managers" sont chargés
de réconcilier l'état désiré du système avec l'état actuel et suivents tous une
architecture semblable qui sera expliqué plus en détails au chapitre XYZ.

La solution est composée de X composants:
+ `supervisor`
+ `system-manager`
+ `network-manager`
+ `container-manager`
+ `storage-manager`










= Conceptuelle

== Utilisation de la solution

L'administrateur du système crée un fichier de configuration, dans le cas le
plus simple, celui-ci contient:
- la configuration réseau



== Composants logiciel

=== Superviseur

Le superviseur est le premier exécutable appelé par le noyeau Linux, il doit,
dans l'ordre:
+ monter l'ensemble des pseudo-FS (càd `/dev`, `/sys`, etc.)
+ monter la configuration (sous `/etc/containers/config`)

Le superviseur est le premier exécutable appelé par le noyeau Linux, il doit:
- monter la configuration

=== System Manager
=== Network Manager
=== Container Manager

== Gestion de l'état

Deux architecture possible:
+ un composant central qui stocke et schedule les réconciliation (pour donner a
    d'autre composants)
+ ou, le composant central ne stoque que la data

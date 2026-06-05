= Conceptuelle

== Utilisation de la solution

L'administrateur du système crée un fichier de configuration, dans le cas le plus simple, celui-ci contient:
- la configuration réseau



== Composants logiciel

=== Superviseur

Le superviseur est le premier exécutable appelé par le noyeau Linux, il doit, dans l'ordre:
+ monter l'ensemble des pseudo-FS (càd `/dev`, `/sys`, etc.)
+ monter la configuration (sous `/etc/containers/config`)

Le superviseur est le premier exécutable appelé par le noyeau Linux, il doit:
- monter la configuration

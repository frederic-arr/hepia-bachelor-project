#import "../lib.typ": *

= Résultats

// TODO

// TODO: Retrospective?

== Plan fonctionnel

Les scénarios de validation présentés au chapitre précédent démontrent la
correction fonctionnelle du système dans trois configurations représentatives,
et la comparaison avec NixOS et Talos Linux établit que le système présente une
empreinte mémoire et une taille d'image nettement inférieures à celles de Talos
Linux, pour un périmètre fonctionnel volontairement restreint au déploiement
unitaire.

Le système fonctionne également sur un Raspberry Pi 1B, sans qu'aucune
adaptation notable n'ait été nécessaire pour ce matériel. Bien que le projet
visait a être déployé sur des Raspberry Pi moderne, le fait qu'il soit
suffisement léger pour fonctionner sur un modèle aussi vieux est vraiment bien.

D'un point de vue fonctionnel, la valeur ajoutée du système réside dans sa
rapidité de démarrage et dans sa faible empreinte mémoire, deux propriétés
mesurées au chapitre précédent et confirmées par la comparaison avec les
solutions existantes. Cette rapidité permet un déploiement quasi immédiat d'un
conteneur, aussi bien depuis une image ISO que depuis une installation sur
disque.

Le système présente toutefois des limites fonctionnelles. Deux fonctionnalités
identifiées lors de la planification ne sont pas implémentées: la gestion de
tâches planifiées ("job scheduling") ainsi que le monitoring du système.
L'absence de tâches planifiées fait que il est plus compliqué d'exécuter des
routines régulières (par exemple backups), tandis que l'absence de monitoring
complique la détection et le diagnostic d'une défaillance. En outre, seul une
partie des options de configuration du réseau et des conteneurs ont été
implémenté, principalement dans un soucis de temps.

== Résultats académiques <results-academic>

L'ensemble des objectifs techniques formulés dans l'énoncé du sujet sont remplis
par le système développé. Les résultats fonctionnels présentés à la section
précédente, en particulier la validation des trois scénarios de bout en bout et
l'atteinte de l'objectif de légèreté par comparaison avec Talos Linux,
établissent cette conformité. Les trois fonctionnalités non implémentées ne
figurent pas parmi les objectifs centraux de l'énoncé et sont écartées afin de
concentrer l'effort sur la robustesse du système, jugée prioritaire. Au regard
de l'ensemble de ces éléments, les objectifs de l'énoncé du sujet sont
considérés comme atteints.

La planification initiale du travail de diplôme est établie sous la forme d'un
diagramme de Gantt, structuré par semaine sur l'ensemble de la durée du projet.
La #figure-num-ref(<plan-gantt>) présente cette planification, complétée a
posteriori par l'avancement effectif de chaque tâche.

#{
    set page(flipped: true)
    include "../diagrams/plan-gantt.typ"
}

La planification reste fidèle à l'avancement effectif jusqu'au début du mois de
juillet. À partir de cette période, la correspondance entre la planification et
l'avancement réel devient plus approximative, l'avancement constaté jusqu'alors
laissant présager une marge suffisante pour respecter les délais fixés; le
rythme de travail est alors volontairement réduit.

La tâche "Test et validation" figurant sur le diagramme ne recouvre pas
l'ensemble des activités de test menées durant le projet, le système faisant
l'objet de tests continus dès les premières étapes de son développement,
notamment au moyen des tests unitaires et d'intégration décrits au chapitre
précédent. Cette tâche correspond spécifiquement à la phase de recherche
exhaustive de cas limites ("edge cases") et de validation de bout en bout, menée
une fois les composants principaux du système stabilisés. Cette tâche est
avancée et prolongée au-delà de la durée prévue, recouvrant une partie de la
période initialement réservée à l'amélioration du code, cette dernière tâche
ayant elle-même été anticipée. Ce chevauchement s'explique par la nécessité de
consolider certains composants avant que les tests de bout en bout ne puissent
être exécutés de manière fiable. La phase de recherche de cas limites est
néanmoins menée à son terme avant la fin du projet, avec un léger décalage par
rapport à la période initialement planifiée.

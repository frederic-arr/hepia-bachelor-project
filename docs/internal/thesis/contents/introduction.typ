#import "../lib.typ"

#show heading.where(level: 2): set heading(outlined: false)

= Introduction
// == Contexte
// == Problématique
// == Objectifs
// == Structure du document

Le déploiement d'applications conteneurisées s'appuie désormais largement sur
des approches déclaratives, en particulier dans les environnements orchestrés.
En revanche, la configuration du système hôte reste souvent prise en charge par
des outils impératifs distincts, ce qui introduit une rupture entre la gestion
de l'infrastructure sous-jacente et celle des applications.

Cette séparation limite l'automatisation complète du déploiement et complique la
reproductibilité, en particulier dans les contextes de déploiement unitaire, où
les solutions conçues pour des environnements clusterisés peuvent s'avérer peu
adaptées.

Dans ce contexte, ce mémoire analyse les besoins auxquels doit répondre un
système d'exploitation destiné au déploiement d'applications conteneurisées sur
hôte unique, évalue dans quelle mesure les solutions existantes y répondent,
puis propose, lorsqu'aucune d'entre elles ne satisfait de manière adéquate aux
exigences identifiées, une architecture de haut niveau adaptée à ce contexte.

== Cadre du travail

Le présent travail s'inscrit dans le cadre du projet de semestre, qui constitue
une étape préalable au projet de Bachelor menant à l'obtention du Bachelor of
Science en Informatique et systèmes de communication, avec une orientation en
informatique logicielle, à la Haute École du paysage, d'ingénierie et
d'architecture de Genève (HEPIA / HES-SO Genève). Sa réalisation s'effectue en
parallèle des enseignements, à raison de cinq heures par semaine durant cinq
mois, d'octobre à la fin mars.

== Définitions et mots-clés

Le présent travail s'inscrit dans le domaine de la conteneurisation. Il s'agit
d'une approche qui consiste à regrouper une application et ses dépendances sous
la forme d'une image de conteneur /* @bib-cncf-glossary-containerization */,
puis à l'exécuter de manière isolée du reste du système dans un conteneur
/* @bib-cncf-glossary-containers */ au moyen d'un environnement d'exécution
spécialisé /* @bib-cncf-glossary-runtime @bib-redhat-containers-vm */.

Le terme "charge applicative" désigne ici une application conteneurisée, qu'elle
soit composée d'un seul composant ou de plusieurs services.

En outre, le terme "déploiement unitaire" désigne ici un déploiement réalisé sur
un hôte unique, géré individuellement, sans orchestration multinœuds. Cette
expression est utilisée ici comme équivalent fonctionnel des notions de
_single-node deployment_ ou de _single-machine deployment_ rencontrées dans la
documentation technique.

Enfin, une configuration déclarative désigne ici une configuration dans laquelle
l'état souhaité du système est décrit explicitement, sans détailler la suite
d'actions à exécuter pour l'atteindre /* @bib-declarative-config */.

== Méthodologie

Le travail a été mené en plusieurs étapes. Une première phase a consisté à
formaliser les besoins à partir de cas d'usage concrets issus de la pratique.
Une deuxième phase a porté sur l'identification et l'évaluation de solutions
existantes au regard de ces besoins. Lorsqu'aucune solution ne répondait de
manière satisfaisante aux exigences identifiées, une proposition conceptuelle a
été élaborée, puis traduite sous la forme d'une architecture de haut niveau.

L'analyse des cas d'usage repose principalement sur un retour d'expérience issu
de la pratique. Le reste du document s'appuie sur des sources externes,
principalement la documentation technique officielle des solutions étudiées,
ainsi que sur des articles et billets techniques indépendants comparant
différentes approches. Lorsque cela était possible, les informations issues de
ces différentes sources ont été croisées.

Une assistance par outil d'intelligence artificielle générative a été utilisée
pour la reformulation de certains passages et pour l'amélioration stylistique de
sections rédactionnelles. En revanche, ces outils n'ont eu aucune influence sur
la structure générale du rapport, les recherches, l'analyse des solutions, les
choix conceptuels ou les décisions architecturales.

== Structure du document

Ce mémoire est structuré en cinq chapitres principaux, suivis d'une discussion
et d'une conclusion. Le premier chapitre porte sur l'analyse des besoins,
formalisant les exigences issues de cas d'usage concrets. Le deuxième chapitre
présente un état de l'art des solutions existantes et procède à leur évaluation
pour en vérifier l'adéquation aux besoins identifiés. Le troisième chapitre
expose la solution conceptuelle proposée, en présentant ses principes
directeurs, tels que la déclarativité et l'automatisation. Le quatrième chapitre
synthétise les briques technologiques nécessaires à son implémentation, telles
que les mécanismes d'isolation et de sécurité du noyau Linux. Le cinquième
chapitre définit une architecture générale de haut niveau intégrant ces
technologies. La discussion revient sur les limites de la solution et les
perspectives d'extension.

#import "../lib.typ": *

= Conclusion
/*
- Rappel du but du travail
- Rappel du fait que c'est un sujet personnel
- On a démontré que au final, cette solution c'est pas juste un jouet mais c'est
    quelque chose de vraiment utilisable (via le fait qu'on déploie une vrai app
    3 tier dessus)
- Et vraiment utile!
- J'ai eu beaucoup de plaisir a découvrir des choses
- J'ai pu mettre en application les éléments de cours lié à la conteneurisation
    dans la pratique ce qui m'a permis de vraiment mieux comprendre
- Je conaissait aussi Nix l'OS mais j'avais moins utilisé le système de build et
    je suis agréablement surpris (bon a par la vitesse)
- La consommation de RAM et la rapidité dépasse largement ce que j'espérais
- Je suis un peu déçu de ne pas pu avoir implémenté le logging/monitoring et ça
    sera la priorité pour la suite de ce projet
- J'ai beaucoup d'idée pour ce projet
- Et je compte absolument continuer a travailler dessus après
*/

Ce travail répond à une problématique identifiée à titre personnel par:
l'absence, parmi les solutions existantes, d'un système d'exploitation alliant
déclarativité continue, intégration native de la conteneurisation et légèreté
suffisante pour des déploiements unitaires. Le travail de semestre ayant précédé
ce travail de diplôme avait établi qu'aucune des solutions examinées, telles que
NixOS ou Talos Linux, ne répondait simultanément à ces trois exigences. Le
système développé au cours du présent travail comble ce vide, en s'appuyant sur
les conclusions architecturales issues de cette analyse préalable.

La solution proposée est administré selon un modèle d'administration entièrement
déclaratif, reposant sur un unique fichier de configuration, ce qui garantit une
prise en main simple et une intégration naturelle dans un contexte GitOps. Cette
déclarativité repose, sur le plan conceptuel, sur un modèle de ressource
homogène, dont la réconciliation est prise en charge par un orchestrateur
centralisé, ce choix architectural privilégiant la coordination des dépendances
entre ressources, au détriment d'une flexibilité de planification jugée
secondaire au regard du périmètre du système. L'implémentation de cette
architecture, détaillée au chapitre correspondant, repose sur Rust pour
l'ensemble des composants et sur Nix pour la chaîne de build, ce dernier choix
garantissant la reproductibilité des artefacts produits.

Les résultats présentés au chapitre précédent établissent que le système dépasse
le stade d'une simple preuve de concept. Le déploiement d'une application
complexe réelle, comprenant une base de données, plusieurs services backend et
un service web, ainsi que la validation de la persistance des données à travers
un redémarrage, démontrent une robustesse fonctionnelle suffisante pour des cas
d'usage réels, et non uniquement pour des scénarios de démonstration simplifiés.
Les mesures effectuées confirment par ailleurs l'atteinte de l'objectif de
légèreté fixé en introduction: le système démarre un conteneur en moins de 5.1
secondes lorsqu'une image doit être téléchargée, et ne requiert que 160 Mio de
mémoire pour exécuter un serveur web minimal, un seuil sensiblement inférieur à
celui des solutions comparables. Au regard de l'ensemble de ces éléments, les
objectifs techniques fixés par l'énoncé du sujet sont considérés comme atteints.

Ce travail a également permis de mettre en application, dans un contexte
concret, les notions de conteneurisation abordées durant la formation,
contribuant à une compréhension plus approfondie de leur fonctionnement interne.
Il a en outre permis d'approfondir l'usage de Nix comme système de build,
jusqu'alors moins pratiqué que l'usage de NixOS en tant que distribution, et
dont l'apport en matière de reproductibilité s'est révélé concluant, en dépit
des temps de compilation élevés discutés au chapitre précédent.

Les limites identifiées, notamment l'absence d'un mécanisme de journalisation et
de supervision, ainsi que les perspectives d'évolution détaillées précédemment,
indiquent que le développement du système est appelé à se poursuivre au-delà du
cadre de ce travail de diplôme. Le périmètre volontairement restreint retenu
ici, celui du déploiement unitaire, constitue un socle sur lequel les extensions
envisagées, qu'il s'agisse de l'observabilité, du stockage ou d'une intégration
plus poussée avec Kubernetes, pourront être construites sans remettre en cause
les choix architecturaux fondamentaux opérés dans ce travail.

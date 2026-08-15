#import "../lib.typ": *

= Conclusion
Ce travail répond à une problématique identifiée à titre
personnel#sym.space.narrow: l'absence, parmi les solutions existantes, d'un
système d'exploitation alliant déclarativité continue, intégration native de la
conteneurisation et légèreté suffisante pour des déploiements modestes. Le
travail de semestre ayant précédé ce travail de diplôme avait établi qu'aucune
des solutions examinées, telles que NixOS ou Talos Linux, ne répondait
simultanément à ces trois exigences. Le système développé au cours du présent
travail comble ce vide, en s'appuyant sur les conclusions architecturales issues
de cette analyse préalable.

La solution proposée est administrée selon un modèle d'administration
entièrement déclaratif, reposant sur un unique fichier de configuration. Le
système n'expose aucun accès interactif de type SSH ou console d'administration
locale#sym.space.narrow; l'API constitue l'unique surface d'administration, tant
pour la soumission initiale de la configuration que pour toute modification
ultérieure, ce qui garantit une prise en main simple et une intégration
naturelle dans un contexte GitOps. Cette déclarativité repose sur un modèle de
ressource homogène, dont la réconciliation est prise en charge par un
orchestrateur centralisé, ce choix architectural privilégiant la coordination
des dépendances entre ressources, au détriment d'une flexibilité de
planification jugée secondaire au regard du périmètre du système.
L'implémentation de cette architecture, détaillée au #chapter-full-ref(
    <ch:implementation>,
), repose sur Rust pour l'ensemble des composants et sur Nix pour la chaîne de
build, ce dernier choix garantissant la reproductibilité des artefacts produits.

Les résultats présentés au #chapter-full-ref(<ch:results>) établissent que le
système dépasse le stade d'une simple preuve de concept. Le déploiement d'une
application complexe réelle, ainsi que la validation de la persistance des
données à travers un redémarrage, démontrent une robustesse fonctionnelle
suffisante pour des cas d'usage réels, et non uniquement pour des scénarios de
démonstration simplifiés. Les mesures effectuées dans le #chapter-full-ref(
    <ch:validation:bench>,
) confirment par ailleurs l'atteinte de l'objectif de légèreté fixé en
introduction#sym.space.narrow: le système démarre un conteneur en moins de 5.1
secondes lorsqu'une image doit être téléchargée, et ne requiert que 160 MiB de
mémoire pour exécuter un serveur web minimal, un seuil sensiblement inférieur à
celui des solutions comparables. Au regard de l'ensemble de ces éléments, les
objectifs techniques fixés par l'énoncé du sujet sont considérés comme atteints.

Ce travail a également permis de mettre en application, dans un contexte
concret, les notions de conteneurisation abordées durant la formation,
contribuant à une compréhension plus approfondie de leur fonctionnement interne.
Il a en outre permis d'approfondir l'usage de Nix comme système de build, dont
l'apport en matière de reproductibilité s'est révélé concluant, en dépit des
temps de compilation élevés discutés au #chapter-full-ref(<ch:results:limits>).

Le périmètre volontairement restreint retenu ici, celui du déploiement modeste
et sans clustering, constitue une base sur laquelle les extensions envisagées,
qu'il s'agisse de l'observabilité, du stockage ou d'une intégration plus poussée
avec la conteneurisation, pourront être construites sans remettre en cause les
choix architecturaux fondamentaux faits dans ce travail. S'agissant d'un projet
avant tout personnel, le développement du système est appelé à se poursuivre
au-delà du cadre de ce travail de diplôme, avec pour objectifs immédiats
d'implémenter les fonctionnalités relatives à l'observabilité et au stockage.

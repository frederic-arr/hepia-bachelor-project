#import "/lib/templates/single-page-common/lib.typ"

#show: lib.single-page.with(
    anchor: "Résumé",
    title: [Résumé],
)

#lorem(300)

/*
L'approche DevOps et la conteneurisation sont devenu omniprésentes dans le
déploiement d'application et d'infrastructure. Pourtant, les systèmes
d'exploitation sous-jacents sont restés en retrait de cette évolution: les
distributions généralistes embarquent bien plus de fonctionnalités que
nécessaire sans répondre nativement aux besoins de ce contexte, tandis que les
systèmes spécialisés s'intègrent étroitement à Kubernetes, sans fournir de
solution indépendante de tout orchestrateur. Ce travail reprend les conclusion
du projet de semestre afin de concevoir, implémenter, et évaluer un système
d'exploitation minimaliste, n'embarquant que le noyau Linux, une runtime de
conteneur avec Podman, et une couche de configuration système déclarative et
piloté par API développé durant ce travail. Le but étant de fournir un OS léger,
sécurisé, et deployable sur des systèmes disposant de peu de resource tout en
permettant l'adoption des méthodologies DevOps. #highlight(text(
    fill: red,
    weight: "bold",
)[
    TODO: suite du résumé une fois le projet terminé
])
*/

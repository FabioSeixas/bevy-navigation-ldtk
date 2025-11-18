1. Ver sobre o SpatialIndex em https://bevy.org/examples/ecs-entity-component-system/observers/
2. Movimento em diagonal
3. Collision. 
-  Se quero entrar em um GridPosition, preciso primeiro saber se esse não possui um dono.
Se não tiver, posso seguir e me tornar o novo dono daquele GridPosition. Nesse momento também deixo de ser o Owner
do GridPosition anterior. Se o GridPosition que quero ir já está ocupado, devo buscar outro adjacente ou próximo.
- Ao me tornar o dono de um GridPosition, esse obtém um Owned Component apontando para mim (Entity). Esse é o marcador
sinalizando que ele não está livre

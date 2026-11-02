# Ambiente Kubernetes da organização

Este manifesto sobe a `organization-api`, PostgreSQL e RabbitMQ no namespace
compartilhado `magnificient`. Assim, `client-api` e `producer-api` poderão usar os
mesmos serviços internos `postgres:5432` e `rabbitmq:5672` ao serem implantados
nesse namespace. De outro namespace, use `postgres.magnificient.svc.cluster.local`
e `rabbitmq.magnificient.svc.cluster.local`.

Banco e broker usam volumes persistentes. Os serviços ficam internos ao cluster;
a API fica acessível por `ClusterIP` na porta `8181`.

## Build e aplicação

Na raiz do repositório, construa a imagem com a tag usada pelo manifesto:

```sh
docker build -f deploy/docker/organizacao-api/Dockerfile \
  -t magnificient/organizacao-api:0.1.1 .
```

Se estiver usando um cluster local, importe a imagem conforme o cluster usado
(por exemplo, `kind load docker-image` ou `minikube image load`). Para um cluster
remoto, publique a imagem em um registry e ajuste `image` em `stack.yaml`.

Antes de aplicar, troque `change-me-before-production` no Secret
`magnificient-middleware-credentials` por credenciais próprias. Atualize a senha
do RabbitMQ e a senha embutida em
`APP__RABBITMQ__RABBITMQ_URL` juntas. Os valores no manifesto servem somente para
um ambiente local de desenvolvimento; para produção, prefira criar o Secret fora
do repositório ou usar um gerenciador de segredos integrado ao cluster.

```sh
kubectl apply -f deploy/kubernetes/organizacao/stack.yaml
kubectl -n magnificient get pods,services,pvc
```

Para acessar a API localmente:

```sh
kubectl -n magnificient port-forward service/organization-api 8181:8181
```

Então consulte `http://127.0.0.1:8181/health`.

## Configuração

O `ConfigMap` fornece as variáveis `APP__...` não secretas. O `Secret` fornece a
senha do PostgreSQL e a URL do RabbitMQ. A aplicação usa o separador `__` para
mapear essas variáveis à configuração hierárquica e agora permite iniciar sem o
arquivo TOML dentro da imagem.

As migrações SQL são aplicadas pela API durante a inicialização. PostgreSQL e
RabbitMQ usam `StatefulSet` com volumeClaimTemplates; o cluster precisa ter uma
StorageClass padrão ou uma política de provisionamento compatível com os PVCs.

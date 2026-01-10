[![crates.io](https://img.shields.io/badge/crates.io-0.1.0--pre17-orange)](https://crates.io/crates/hu/0.1.0-pre17)

aws sso login # if needed check if needed

aws eks update-kubeconfig --name prod-eks --region us-east-1 # from --env prod, alternative --env dev --env stg

kubectl get pod --namespace cms | grep web | awk '{print $1}' # web pattern from --type command

number pods 1,2,3 ... passed via --pod 1 switch

kubectl exec -it eks-cms-web-deployment-5ffb745bd-bs6pv -n cms -- bash

But I want a "prod-web-{number assigned earler} $ " PS prompt on my container bash

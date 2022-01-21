k8s_openapi::k8s_if_ge_1_20! {
    crate::condition!(k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition[core]);
}

crate::condition!(k8s_openapi::api::batch::v1::JobCondition[probe]);
crate::condition!(k8s_openapi::api::apps::v1::StatefulSetCondition);

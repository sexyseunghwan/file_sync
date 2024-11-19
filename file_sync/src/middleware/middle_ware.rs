use crate::common::*;

#[derive(Debug, Clone)]
pub struct CheckIp {
    pub master_address: Arc<Vec<String>>
}

impl CheckIp {
    pub fn new(master_address: Vec<String>) -> Self {
        CheckIp {
            master_address: Arc::new(master_address)
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CheckIp
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CheckIpMiddleware<S>;
    type InitError = ();
    type Future = FuterReady<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckIpMiddleware {
            service,
            master_address: self.master_address.clone()
        })
    }
}

pub struct CheckIpMiddleware<S> {
    service: S,
    master_address: Arc<Vec<String>>,
}

impl<S, B> Service<ServiceRequest> for CheckIpMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }
    
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let client_ip = req.peer_addr().map(|addr| addr.ip().to_string()).unwrap_or_default();
        
        if self.master_address.contains(&client_ip) {
            Box::pin(self.service.call(req))
        } else {
            Box::pin(async move {
                error!("This IP address is not allowed to access. : {}", client_ip);
                Err(actix_web::error::ErrorForbidden("IP not allowed"))
            })
        }
    }
}
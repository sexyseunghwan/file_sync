use crate::common::*;


#[derive(Debug, new)]
pub struct CheckIp {
    pub master_address: Vec<String>
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
        ok(CheckIpMiddleware { service })
    }
}

pub struct CheckIpMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckIpMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    //type Future = S::Future;
    type Future = std::pin::Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    #[doc = "내부 서비스의 준비 상태를 전달"]
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx) 
    }
    
    #[doc = "docs"]
    fn call(&self, req: ServiceRequest) -> Self::Future {
    
        let client_ip = req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_default();
        
        /* 허용하고자 하는 IP 목록 - master ip 만 받아야 한다. */ 
        let allowed_ips = vec!["192.168.8.77"];
        
        if allowed_ips.contains(&client_ip.as_str()) {
            Box::pin(self.service.call(req))
        } else {
            return Box::pin(async move {
                Err(actix_web::error::ErrorBadRequest("not allowed"))
            });
        }
    }
}
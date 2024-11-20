use crate::common::*;


#[doc = "수신 요청의 IP 주소를 확인하는 IP 기반 액세스 제어용 미들웨어"]
#[derive(Debug, Clone)]
pub struct CheckIp {
    
    /* 허용된 IP 주소 목록  */
    pub master_address: Arc<Vec<String>>
}

impl CheckIp {

    #[doc = "'CheckIp' 미들웨어의 새 인스턴스를 만들어주는 함수"]
    /// # Arguments
    /// * `master_address` - 허용된 IP 주소를 나타내는 문자열 벡터.
    /// 
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


    #[doc = "서비스 'S'를 새로운 '체크아이피 미들웨어'로 감싸 IP 주소를 기반으로 요청을 가로채고 처리가능."]
    /// # Arguments
    /// * `service` - 미들웨어가 감싸고 있는 서비스
    /// 
    /// # Returns
    /// Self::Future - 'CheckIpMiddleware' 또는 초기화 오류로 반환되는 Future 인스턴스
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

    #[doc = "서비스가 요청을 처리할 준비가 되었는지 확인해주는 함수."]
    /// # Arguments
    /// * `cx` - 비동기 작업을 촉진하기 위한 컨텍스트
    /// 
    /// # Returns
    /// Poll<Result<(), Self::Error>> - 기본 서비스가 요청을 처리할 준비가 되었는지 여부
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }
    

    #[doc = "수신되는 '서비스 요청'을 처리한다. 클라이언트의 IP가 허용되면 요청을 내부 서비스로 전달하고, 허용되지 않으면 403 금지 오류로 응답."]
    /// # Arguments
    /// * `req` - 처리할 수신 HTTP 요청
    /// 
    /// # Returns
    /// * Self::Future - 이 요청을 처리하는 과정을 나타내는 Future 타입으로, IP가 허용되지 않을 경우 응답 또는 오류가 발생한다.
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
use serde::{Deserialize, Serialize};



#[derive(Debug,  Deserialize , Serialize)]

pub struct AcquireRequest{
    pub lock_id : String , 
    pub client_id : String , 
    pub  time_to_live : u64
}

#[derive(Serialize ,  Debug)]
pub enum AcquireResponse{
    Granted {
         lease_id : String , 
         expires_at : String
    } , 
    Queued {
         position : usize , 
         estimated_wait : u64
    }
    , Error{
        error_type: String,
        message: String,
    }
}

#[derive(Deserialize , Debug )]
pub struct ReleaseRequest{
    pub lock_id : String , 
    pub lease_id : String , 
    pub client_id : String , 
}
#[derive(Serialize , Debug )]
pub enum ReleaseResponse{
    Success ,
     Error{
        error_type: String,
        message: String,
    } 
}

#[derive(Deserialize , Debug)]
pub struct RenewRequest{
    pub lock_id : String , 
    pub client_id : String , 
    pub lease_id : String , 
    pub time_to_live : u64
}

#[derive(Serialize , Debug)]
pub enum RenewResponse{
    Success ,
     Error{
        error_type: String,
        message: String,
    }
}
#[derive(Serialize , Debug)]
pub enum StatusResponse{
    InUse{
        client_id : String ,
        expires_at : String,
        lease_id : String, 
        queue_length : usize , 
        created_at : String , 

    } ,
    Free
}

#[derive(Serialize , Debug)]
pub struct ApiError{
    pub error : String , 
    pub message : String , 
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details : Option<String>
}

impl ApiError{
    pub fn not_found (message:&str) -> Self{
        Self { error: "NotFound".to_string(), message: message.to_string(), details: None }
    }

    pub fn conflict(message : &str) -> Self{
        Self { error: "Conflict".to_string(), message: message.to_string(), details: None }
    }
}
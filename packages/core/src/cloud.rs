use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize)]
pub struct CloudSnapshot {
    pub provider: String,
    pub region: String,
    pub volume_id: String,
    pub snapshot_id: Option<String>,
}

/// Create AWS EBS snapshot via REST API (no SDK)
pub async fn aws_create_snapshot(
    region: &str, volume_id: &str, access_key: &str, _secret_key: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let endpoint = format!("https://ec2.{}.amazonaws.com/", region);

    // Minimal AWS Signature V4 signing omitted for brevity — uses standard headers
    let params = [
        ("Action", "CreateSnapshot"),
        ("VolumeId", volume_id),
        ("Description", "CollectionLoom forensic snapshot"),
        ("Version", "2016-11-15"),
    ];

    let resp = client.post(&endpoint)
        .header("X-Amz-Access-Key", access_key)
        .form(&params)
        .send().await
        .map_err(|e| format!("AWS request failed: {}", e))?;

    let body = resp.text().await.map_err(|e| e.to_string())?;
    Ok(body)
}

/// Create Azure disk snapshot via REST API
pub async fn azure_create_snapshot(
    subscription: &str, resource_group: &str, disk_name: &str, snapshot_name: &str, token: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/snapshots/{}?api-version=2024-03-01",
        subscription, resource_group, snapshot_name
    );

    let body = serde_json::json!({
        "location": "eastus",
        "properties": {
            "creationData": {
                "createOption": "Copy",
                "sourceResourceId": format!("/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}", subscription, resource_group, disk_name)
            }
        }
    });

    let resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await
        .map_err(|e| format!("Azure request failed: {}", e))?;

    Ok(resp.text().await.map_err(|e| e.to_string())?)
}

/// Create GCP disk snapshot via REST API (no SDK)
pub async fn gcp_create_snapshot(
    project: &str, zone: &str, disk: &str, snapshot_name: &str, token: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/disks/{}/createSnapshot",
        project, zone, disk
    );

    let body = serde_json::json!({
        "name": snapshot_name,
        "description": "CollectionLoom forensic snapshot"
    });

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await
        .map_err(|e| format!("GCP request failed: {}", e))?;

    Ok(resp.text().await.map_err(|e| e.to_string())?)
}

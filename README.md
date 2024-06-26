<h1 align="center" style="border-bottom: none;">📊 Ceph RBD Metrics</h1>
<p align="center">Prometheus exporter for more detailed metrics of Ceph RBD</p>

<br />


### ✨ Features
Provides per RBD Image metrics such as disk usage, quota size, size of objects, and number of objects.

<br />

### 💠 Installation
1. **Get the binary of ceph-rbd-metrics.**
   ```sh
   git clone https://github.com/AsPulse/ceph-rbd-metrics
   cargo build --release
   ```
   or
   ```sh
   # currently linux/amd64, linux/arm64 supported
   docker pull harbor.aspulse.dev/ceph-rbd-metrics/ceph-rbd-metrics:v0.2.0
   ```

  
2. **Set the environment variables.**
   <dl>
     <dt>PORT</dt>
     <dd>The port number eported ceph-rbd-metrics at. Default as 3000.</dd>
     <dt>RUST_LOG</dt>  
     <dd>Log level. `INFO` recommended, but `TRACE` is helpful for debugging.</dd>
     <dt>CEPH_API_ENDPOINT</dt>  
     <dd>The endpoint url of Ceph RESTful API.  Usually, it is as same as dashboard url.</dd>
     <dt>CEPH_API_USERNAME</dt>  
     <dd>The username which the exporter uses for API authorization.</dd>
     <dt>CEPH_API_PASSWORD</dt>  
     <dd>The password which the exporter uses for API authorization.</dd>
   </dd>   

   For the `CEPH_API_USERNAME` and `CEPH_API_PASSWORD`, The username and password you normally use to log in to the dashboard will work, as long as you have sufficient permissions.  
   You can specify its by not only environment variables, also `.env` file.

3. **Run and have fun!**

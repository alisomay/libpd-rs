doc:
  cargo doc && ruby -run -e httpd ./target/doc -p 4000 & sleep 4 && open http://localhost:4000/libpd_rs
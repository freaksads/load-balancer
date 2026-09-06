{...}: {
  languages.rust = {
    enable = true;
  };

  services.nginx = {
    enable = true;

    httpConfig = ''
      server {
        listen 8001;

        location / {
          return 200 "Backend 1\n";
        }

        location = /health {
          access_log off;
          default_type text/plain;

          return 200 "OK\n";
        }
      }

      server {
        listen 8002;

        location / {
          return 200 "Backend 2\n";
        }

        location = /health {
          access_log off;
          default_type text/plain;

          return 200 "OK\n";
        }
      }

      server {
        listen 8003;

        location / {
          return 200 "Backend 3\n";
        }

        location = /health {
          access_log off;
          default_type text/plain;

          return 200 "OK\n";
        }
      }
    '';
  };
}

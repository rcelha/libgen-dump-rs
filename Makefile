container_cmd=podman

.PHONY: help
help:
	@echo "TODO"

data:
	mkdir $(PWD)/data

data/libgen.rar: data
	wget http://libgen.rs/dbdumps/libgen.rar -O $(PWD)/data/libgen.rar

data/libgen.sql: data
	bsdtar -xvf $(PWD)/data/libgen.rar --directory=$(PWD)/data/

wait-for:
	wget -O wait-for https://raw.githubusercontent.com/eficode/wait-for/v2.2.3/wait-for
	chmod +x wait-for

.PHONY: run-mysql
run-mysql:
	$(container_cmd) run --rm \
		-d \
		--name libgen-mysql \
		-p 3306:3306 \
		-e MYSQL_ROOT_PASSWORD=1234 \
		mysql

.PHONY: e2e
e2e: wait-for run-mysql data/libgen.rar data/libgen.sql
	sleep 60
	$(PWD)/wait-for localhost:3306 -- echo mysql is fine
	$(container_cmd) exec -i libgen-mysql sh -c 'exec mysql -uroot -p1234 -e "create database libgen"'
	$(container_cmd) exec -i libgen-mysql sh -c 'exec mysql -uroot -p1234 libgen' < data/libgen.sql

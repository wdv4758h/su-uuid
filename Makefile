wheel:
	python3 setup.py bdist_wheel

install: wheel
	pip3 install dist/su_uuid-*.whl --upgrade

test:
	pytest tests/

bench:
	pytest benchmark/

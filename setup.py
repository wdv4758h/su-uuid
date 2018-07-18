# check "setuptools_rust" first
try:
    from setuptools_rust import Binding, RustExtension
except ImportError:
    import subprocess
    import sys
    errno = subprocess.call(
        [sys.executable, '-m', 'pip', 'install', 'setuptools-rust'])
    if errno:
        print("Please install setuptools-rust package")
        raise SystemExit(errno)
    else:
        from setuptools_rust import Binding, RustExtension

# import setuptools here,
# so it can grab the subcommand from setuptools_rust after installation
from setuptools import setup


setup(name='su-uuid',
      version='0.1.0',
      classifiers=[
          'License :: OSI Approved :: Apache Software License',
          'Development Status :: 3 - Alpha',
          'Intended Audience :: Developers',
          'Programming Language :: Python',
          'Programming Language :: Rust',
          'Operating System :: POSIX',
      ],
      rust_extensions=[
          RustExtension('su_uuid', 'Cargo.toml', binding=Binding.PyO3)],
      zip_safe=False)

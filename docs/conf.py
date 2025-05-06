# Project information
project = 'Teleflow'
copyright = '2025, AsymptotaX'
author = 'AsymptotaX'
release = '0.2.1'

# General configuration
extensions = [
    'myst_parser',  # Enable Markdown support
    'sphinx.ext.autodoc',
    'sphinx.ext.viewcode',
]

source_suffix = ['.md']
master_doc = 'index'

# HTML output
html_theme = 'sphinx_rtd_theme'

# Markdown configuration
myst_enable_extensions = [
    'colon_fence',  # Support ::: for directives
    'deflist',      # Definition lists
]
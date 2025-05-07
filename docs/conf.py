# Project information
project = 'Teleflow'
copyright = '2025, AsymptotaX'
author = 'AsymptotaX'
release = '0.3.0'

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
html_theme_options = {
    'logo_only': True,
    'collapse_navigation': False,
    'navigation_depth': 4,
    'style_nav_header_background': 'white',
}

html_logo = "images/logo.png"

# Markdown configuration
myst_enable_extensions = [
    'colon_fence',
    'deflist',
]
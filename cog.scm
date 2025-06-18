(define package-name 'ghost-text)
(define version "0.1.5")

(define dependencies '())

(define dylibs
  '((#:name dylib-name
     #:urls
     (
      (
       #:platform
       "x86_64-windows"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.5/ghost_text.dll")
      (
       #:platform
       "x86_64-macos"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.5/libghost_text.dylib")
      (
       #:platform
       "x86_64-linux"
       #:url
       "https://github.com/nik-rev/ghost-text.hx/releases/download/v0.1.5/libghost_text.so")))))

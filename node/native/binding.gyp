{
  "targets": [
    {
      "target_name": "bbsplus_node",
      "sources": ["src/binding.cc"],
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")",
        "<(module_root_dir)/../../include"
      ],
      "defines": ["NAPI_DISABLE_CPP_EXCEPTIONS"],
      "cflags!": ["-fno-exceptions"],
      "cflags_cc!": ["-fno-exceptions"],
      "variables": {
        "bbsplus_archive%": "<!(node <(module_root_dir)/../scripts/resolve-bbsplus-archive.cjs)"
      },
      "conditions": [
        [
          "OS=='mac'",
          {
            "xcode_settings": {
              "GCC_ENABLE_CPP_EXCEPTIONS": "YES",
              "CLANG_CXX_LIBRARY": "libc++",
              "MACOSX_DEPLOYMENT_TARGET": "11.0",
              "OTHER_LDFLAGS": [
                "-framework Security",
                "-framework CoreFoundation",
                "-Wl,-force_load,<(bbsplus_archive)"
              ]
            }
          }
        ],
        [
          "OS=='linux'",
          {
            "libraries": [
              "-ldl",
              "-lm",
              "-lpthread",
              "-lstdc++",
              "-Wl,--whole-archive",
              "<(bbsplus_archive)",
              "-Wl,--no-whole-archive"
            ]
          }
        ]
      ]
    }
  ]
}

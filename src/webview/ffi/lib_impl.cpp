#include "lib.h"

#if defined(__FreeBSD__) || defined(__NetBSD__) || defined(__OpenBSD__)
#define __BSD__
#endif

#if defined(__APPLE__)
#define WEBVIEW_COCOA
#include "Platform_macOS.c"
#elif defined(__linux__) || defined(__BSD__)
#define WEBVIEW_GTK
#include "Platform_Linux.cpp" // yes, BSD is under the Linux name for Quark, cope.
#endif

extern "C" {

    WEBVIEW_API webview_t webview_create(int debug, void *wnd) {
      auto w = new webview::webview(debug, wnd);
      if (!w->window()) {
        delete w;
        return nullptr;
      }
      return w;
    }

    WEBVIEW_API void webview_destroy(webview_t w) {
      delete static_cast<webview::webview *>(w);
    }

    WEBVIEW_API void webview_run(webview_t w) {
      static_cast<webview::webview *>(w)->run();
    }

    WEBVIEW_API void webview_terminate(webview_t w) {
      static_cast<webview::webview *>(w)->terminate();
    }

    WEBVIEW_API void webview_dispatch(webview_t w, void (*fn)(webview_t, void *),
                                      void *arg) {
      static_cast<webview::webview *>(w)->dispatch([=]() { fn(w, arg); });
    }

    WEBVIEW_API void *webview_get_window(webview_t w) {
      return static_cast<webview::webview *>(w)->window();
    }

    WEBVIEW_API void webview_set_title(webview_t w, const char *title) {
      static_cast<webview::webview *>(w)->set_title(title);
    }

    WEBVIEW_API void webview_set_size(webview_t w, int width, int height,
                                      int hints) {
      static_cast<webview::webview *>(w)->set_size(width, height, hints);
    }

    WEBVIEW_API void webview_navigate(webview_t w, const char *url) {
      static_cast<webview::webview *>(w)->navigate(url);
    }

    WEBVIEW_API void webview_set_html(webview_t w, const char *html) {
      static_cast<webview::webview *>(w)->set_html(html);
    }

    WEBVIEW_API void webview_init(webview_t w, const char *js) {
      static_cast<webview::webview *>(w)->init(js);
    }

    WEBVIEW_API void webview_eval(webview_t w, const char *js) {
      static_cast<webview::webview *>(w)->eval(js);
    }

    WEBVIEW_API void webview_bind(webview_t w, const char *name,
                                  void (*fn)(const char *seq, const char *req,
                                             void *arg),
                                  void *arg) {
      static_cast<webview::webview *>(w)->bind(
          name,
          [=](const std::string &seq, const std::string &req, void *arg) {
            fn(seq.c_str(), req.c_str(), arg);
          },
          arg);
    }

    WEBVIEW_API void webview_unbind(webview_t w, const char *name) {
      static_cast<webview::webview *>(w)->unbind(name);
    }

    WEBVIEW_API void webview_return(webview_t w, const char *seq, int status,
                                    const char *result) {
      static_cast<webview::webview *>(w)->resolve(seq, status, result);
    }

}

#if defined(WEBVIEW_EDGE)
#define WIN32_LEAN_AND_MEAN
#include <shlobj.h>
#include <shlwapi.h>
#include <stdlib.h>
#include <windows.h>

#include "lib.h"
#include "Platform_Windows-WebView2.h"

#ifdef _MSC_VER
#pragma comment(lib, "advapi32.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "shlwapi.lib")
#pragma comment(lib, "user32.lib")
#pragma comment(lib, "version.lib")
#endif

namespace webview {

    namespace detail {

        using msg_cb_t = std::function<void(const std::string)>;

        // Converts a narrow (UTF-8-encoded) string into a wide (UTF-16-encoded) string.
        inline std::wstring widen_string(const std::string& input) {
            if (input.empty()) {
                return std::wstring();
            }
            UINT cp = CP_UTF8;
            DWORD flags = MB_ERR_INVALID_CHARS;
            auto input_c = input.c_str();
            auto input_length = static_cast<int>(input.size());
            auto required_length =
                MultiByteToWideChar(cp, flags, input_c, input_length, nullptr, 0);
            if (required_length > 0) {
                std::wstring output(static_cast<std::size_t>(required_length), L'\0');
                if (MultiByteToWideChar(cp, flags, input_c, input_length, &output[0],
                    required_length) > 0) {
                    return output;
                }
            }
            // Failed to convert string from UTF-8 to UTF-16
            return std::wstring();
        }

        // Converts a wide (UTF-16-encoded) string into a narrow (UTF-8-encoded) string.
        inline std::string narrow_string(const std::wstring& input) {
            if (input.empty()) {
                return std::string();
            }
            UINT cp = CP_UTF8;
            DWORD flags = WC_ERR_INVALID_CHARS;
            auto input_c = input.c_str();
            auto input_length = static_cast<int>(input.size());
            auto required_length = WideCharToMultiByte(cp, flags, input_c, input_length,
                nullptr, 0, nullptr, nullptr);
            if (required_length > 0) {
                std::string output(static_cast<std::size_t>(required_length), '\0');
                if (WideCharToMultiByte(cp, flags, input_c, input_length, &output[0],
                    required_length, nullptr, nullptr) > 0) {
                    return output;
                }
            }
            // Failed to convert string from UTF-16 to UTF-8
            return std::string();
        }

        // Parses a version string with 1-4 integral components, e.g. "1.2.3.4".
        // Missing or invalid components default to 0, and excess components are ignored.
        template <typename T>
        std::array<unsigned int, 4>
            parse_version(const std::basic_string<T>& version) noexcept {
            auto parse_component = [](auto sb, auto se) -> unsigned int {
                try {
                    auto n = std::stol(std::basic_string<T>(sb, se));
                    return n < 0 ? 0 : n;
                }
                catch (std::exception&) {
                    return 0;
                }
                };
            auto end = version.end();
            auto sb = version.begin(); // subrange begin
            auto se = sb;              // subrange end
            unsigned int ci = 0;       // component index
            std::array<unsigned int, 4> components{};
            while (sb != end && se != end && ci < components.size()) {
                if (*se == static_cast<T>('.')) {
                    components[ci++] = parse_component(sb, se);
                    sb = ++se;
                    continue;
                }
                ++se;
            }
            if (sb < se && ci < components.size()) {
                components[ci] = parse_component(sb, se);
            }
            return components;
        }

        template <typename T, std::size_t Length>
        auto parse_version(const T(&version)[Length]) noexcept {
            return parse_version(std::basic_string<T>(version, Length));
        }

        std::wstring get_file_version_string(const std::wstring& file_path) noexcept {
            DWORD dummy_handle; // Unused
            DWORD info_buffer_length =
                GetFileVersionInfoSizeW(file_path.c_str(), &dummy_handle);
            if (info_buffer_length == 0) {
                return std::wstring();
            }
            std::vector<char> info_buffer;
            info_buffer.reserve(info_buffer_length);
            if (!GetFileVersionInfoW(file_path.c_str(), 0, info_buffer_length,
                info_buffer.data())) {
                return std::wstring();
            }
            auto sub_block = L"\\StringFileInfo\\040904B0\\ProductVersion";
            LPWSTR version = nullptr;
            unsigned int version_length = 0;
            if (!VerQueryValueW(info_buffer.data(), sub_block,
                reinterpret_cast<LPVOID*>(&version), &version_length)) {
                return std::wstring();
            }
            if (!version || version_length == 0) {
                return std::wstring();
            }
            return std::wstring(version, version_length);
        }

        // A wrapper around COM library initialization. Calls CoInitializeEx in the
        // constructor and CoUninitialize in the destructor.
        class com_init_wrapper {
        public:
            com_init_wrapper(DWORD dwCoInit) {
                // We can safely continue as long as COM was either successfully
                // initialized or already initialized.
                // RPC_E_CHANGED_MODE means that CoInitializeEx was already called with
                // a different concurrency model.
                switch (CoInitializeEx(nullptr, dwCoInit)) {
                case S_OK:
                case S_FALSE:
                    m_initialized = true;
                    break;
                }
            }

            ~com_init_wrapper() {
                if (m_initialized) {
                    CoUninitialize();
                    m_initialized = false;
                }
            }

            com_init_wrapper(const com_init_wrapper& other) = delete;
            com_init_wrapper& operator=(const com_init_wrapper& other) = delete;
            com_init_wrapper(com_init_wrapper&& other) = delete;
            com_init_wrapper& operator=(com_init_wrapper&& other) = delete;

            bool is_initialized() const { return m_initialized; }

        private:
            bool m_initialized = false;
        };

        // Holds a symbol name and associated type for code clarity.
        template <typename T> class library_symbol {
        public:
            using type = T;

            constexpr explicit library_symbol(const char* name) : m_name(name) {}
            constexpr const char* get_name() const { return m_name; }

        private:
            const char* m_name;
        };

        // Loads a native shared library and allows one to get addresses for those
        // symbols.
        class native_library {
        public:
            explicit native_library(const wchar_t* name) : m_handle(LoadLibraryW(name)) {}

            ~native_library() {
                if (m_handle) {
                    FreeLibrary(m_handle);
                    m_handle = nullptr;
                }
            }

            native_library(const native_library& other) = delete;
            native_library& operator=(const native_library& other) = delete;
            native_library(native_library&& other) = default;
            native_library& operator=(native_library&& other) = default;

            // Returns true if the library is currently loaded; otherwise false.
            operator bool() const { return is_loaded(); }

            // Get the address for the specified symbol or nullptr if not found.
            template <typename Symbol>
            typename Symbol::type get(const Symbol& symbol) const {
                if (is_loaded()) {
                    return reinterpret_cast<typename Symbol::type>(
                        GetProcAddress(m_handle, symbol.get_name()));
                }
                return nullptr;
            }

            // Returns true if the library is currently loaded; otherwise false.
            bool is_loaded() const { return !!m_handle; }

            void detach() { m_handle = nullptr; }

        private:
            HMODULE m_handle = nullptr;
        };

        struct user32_symbols {
            using DPI_AWARENESS_CONTEXT = HANDLE;
            using SetProcessDpiAwarenessContext_t = BOOL(WINAPI*)(DPI_AWARENESS_CONTEXT);
            using SetProcessDPIAware_t = BOOL(WINAPI*)();

            static constexpr auto SetProcessDpiAwarenessContext =
                library_symbol<SetProcessDpiAwarenessContext_t>(
                    "SetProcessDpiAwarenessContext");
            static constexpr auto SetProcessDPIAware =
                library_symbol<SetProcessDPIAware_t>("SetProcessDPIAware");
        };

        struct shcore_symbols {
            typedef enum { PROCESS_PER_MONITOR_DPI_AWARE = 2 } PROCESS_DPI_AWARENESS;
            using SetProcessDpiAwareness_t = HRESULT(WINAPI*)(PROCESS_DPI_AWARENESS);

            static constexpr auto SetProcessDpiAwareness =
                library_symbol<SetProcessDpiAwareness_t>("SetProcessDpiAwareness");
        };

        class reg_key {
        public:
            explicit reg_key(HKEY root_key, const wchar_t* sub_key, DWORD options,
                REGSAM sam_desired) {
                HKEY handle;
                auto status =
                    RegOpenKeyExW(root_key, sub_key, options, sam_desired, &handle);
                if (status == ERROR_SUCCESS) {
                    m_handle = handle;
                }
            }

            explicit reg_key(HKEY root_key, const std::wstring& sub_key, DWORD options,
                REGSAM sam_desired)
                : reg_key(root_key, sub_key.c_str(), options, sam_desired) {
            }

            virtual ~reg_key() {
                if (m_handle) {
                    RegCloseKey(m_handle);
                    m_handle = nullptr;
                }
            }

            reg_key(const reg_key& other) = delete;
            reg_key& operator=(const reg_key& other) = delete;
            reg_key(reg_key&& other) = delete;
            reg_key& operator=(reg_key&& other) = delete;

            bool is_open() const { return !!m_handle; }
            bool get_handle() const { return m_handle; }

            std::wstring query_string(const wchar_t* name) const {
                DWORD buf_length = 0;
                // Get the size of the data in bytes.
                auto status = RegQueryValueExW(m_handle, name, nullptr, nullptr, nullptr,
                    &buf_length);
                if (status != ERROR_SUCCESS && status != ERROR_MORE_DATA) {
                    return std::wstring();
                }
                // Read the data.
                std::wstring result(buf_length / sizeof(wchar_t), 0);
                auto buf = reinterpret_cast<LPBYTE>(&result[0]);
                status =
                    RegQueryValueExW(m_handle, name, nullptr, nullptr, buf, &buf_length);
                if (status != ERROR_SUCCESS) {
                    return std::wstring();
                }
                // Remove trailing null-characters.
                for (std::size_t length = result.size(); length > 0; --length) {
                    if (result[length - 1] != 0) {
                        result.resize(length);
                        break;
                    }
                }
                return result;
            }

        private:
            HKEY m_handle = nullptr;
        };

        inline bool enable_dpi_awareness() {
            auto user32 = native_library(L"user32.dll");
            if (auto fn = user32.get(user32_symbols::SetProcessDpiAwarenessContext)) {
                if (fn(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE)) {
                    return true;
                }
                return GetLastError() == ERROR_ACCESS_DENIED;
            }
            if (auto shcore = native_library(L"shcore.dll")) {
                if (auto fn = shcore.get(shcore_symbols::SetProcessDpiAwareness)) {
                    auto result = fn(shcore_symbols::PROCESS_PER_MONITOR_DPI_AWARE);
                    return result == S_OK || result == E_ACCESSDENIED;
                }
            }
            if (auto fn = user32.get(user32_symbols::SetProcessDPIAware)) {
                return !!fn();
            }
            return true;
        }

        // Enable built-in WebView2Loader implementation by default.
#ifndef WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL
#define WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL 1
#endif

// Link WebView2Loader.dll explicitly by default only if the built-in
// implementation is enabled.
#ifndef WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK
#define WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL
#endif

// Explicit linking of WebView2Loader.dll should be used along with
// the built-in implementation.
#if WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL == 1 &&                                    \
    WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK != 1
#undef WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK
#error Please set WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK=1.
#endif

#if WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL == 1
// Gets the last component of a Windows native file path.
// For example, if the path is "C:\a\b" then the result is "b".
        template <typename T>
        std::basic_string<T>
            get_last_native_path_component(const std::basic_string<T>& path) {
            if (auto pos = path.find_last_of(static_cast<T>('\\'));
                pos != std::basic_string<T>::npos) {
                return path.substr(pos + 1);
            }
            return std::basic_string<T>();
        }
#endif /* WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL */

        template <typename T> struct cast_info_t {
            using type = T;
            IID iid;
        };

        namespace mswebview2 {
            static constexpr IID
                IID_ICoreWebView2CreateCoreWebView2ControllerCompletedHandler{
                    0x6C4819F3, 0xC9B7, 0x4260, 0x81, 0x27, 0xC9,
                    0xF5,       0xBD,   0xE7,   0xF6, 0x8C };
            static constexpr IID
                IID_ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler{
                    0x4E8A3389, 0xC9D8, 0x4BD2, 0xB6, 0xB5, 0x12,
                    0x4F,       0xEE,   0x6C,   0xC1, 0x4D };
            static constexpr IID IID_ICoreWebView2PermissionRequestedEventHandler{
                0x15E1C6A3, 0xC72A, 0x4DF3, 0x91, 0xD7, 0xD0, 0x97, 0xFB, 0xEC, 0x6B, 0xFD };
            static constexpr IID IID_ICoreWebView2WebMessageReceivedEventHandler{
                0x57213F19, 0x00E6, 0x49FA, 0x8E, 0x07, 0x89, 0x8E, 0xA0, 0x1E, 0xCB, 0xD2 };

#if WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL == 1
            enum class webview2_runtime_type { installed = 0, embedded = 1 };

            namespace webview2_symbols {
                using CreateWebViewEnvironmentWithOptionsInternal_t =
                    HRESULT(STDMETHODCALLTYPE*)(
                        bool, webview2_runtime_type, PCWSTR, IUnknown*,
                        ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler*);
                using DllCanUnloadNow_t = HRESULT(STDMETHODCALLTYPE*)();

                static constexpr auto CreateWebViewEnvironmentWithOptionsInternal =
                    library_symbol<CreateWebViewEnvironmentWithOptionsInternal_t>(
                        "CreateWebViewEnvironmentWithOptionsInternal");
                static constexpr auto DllCanUnloadNow =
                    library_symbol<DllCanUnloadNow_t>("DllCanUnloadNow");
            } // namespace webview2_symbols
#endif /* WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL */

#if WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK == 1
            namespace webview2_symbols {
                using CreateCoreWebView2EnvironmentWithOptions_t = HRESULT(STDMETHODCALLTYPE*)(
                    PCWSTR, PCWSTR, ICoreWebView2EnvironmentOptions*,
                    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler*);
                using GetAvailableCoreWebView2BrowserVersionString_t =
                    HRESULT(STDMETHODCALLTYPE*)(PCWSTR, LPWSTR*);

                static constexpr auto CreateCoreWebView2EnvironmentWithOptions =
                    library_symbol<CreateCoreWebView2EnvironmentWithOptions_t>(
                        "CreateCoreWebView2EnvironmentWithOptions");
                static constexpr auto GetAvailableCoreWebView2BrowserVersionString =
                    library_symbol<GetAvailableCoreWebView2BrowserVersionString_t>(
                        "GetAvailableCoreWebView2BrowserVersionString");
            } // namespace webview2_symbols
#endif /* WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK */

            class loader {
            public:
                HRESULT create_environment_with_options(
                    PCWSTR browser_dir, PCWSTR user_data_dir,
                    ICoreWebView2EnvironmentOptions* env_options,
                    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler
                    * created_handler) const {
#if WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK == 1
                    if (m_lib.is_loaded()) {
                        if (auto fn = m_lib.get(
                            webview2_symbols::CreateCoreWebView2EnvironmentWithOptions)) {
                            return fn(browser_dir, user_data_dir, env_options, created_handler);
                        }
                    }
#if WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL == 1
                    return create_environment_with_options_impl(browser_dir, user_data_dir,
                        env_options, created_handler);
#else
                    return S_FALSE;
#endif
#else
                    return ::CreateCoreWebView2EnvironmentWithOptions(
                        browser_dir, user_data_dir, env_options, created_handler);
#endif /* WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK */
                }

                HRESULT
                    get_available_browser_version_string(PCWSTR browser_dir,
                        LPWSTR* version) const {
#if WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK == 1
                    if (m_lib.is_loaded()) {
                        if (auto fn = m_lib.get(
                            webview2_symbols::GetAvailableCoreWebView2BrowserVersionString)) {
                            return fn(browser_dir, version);
                        }
                    }
#if WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL == 1
                    return get_available_browser_version_string_impl(browser_dir, version);
#else
                    return S_FALSE;
#endif
#else
                    return ::GetAvailableCoreWebView2BrowserVersionString(browser_dir, version);
#endif /* WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK */
                }

            private:
#if WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL == 1
                struct client_info_t {
                    bool found = false;
                    std::wstring dll_path;
                    std::wstring version;
                    webview2_runtime_type runtime_type;
                };

                HRESULT create_environment_with_options_impl(
                    PCWSTR browser_dir, PCWSTR user_data_dir,
                    ICoreWebView2EnvironmentOptions* env_options,
                    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler
                    * created_handler) const {
                    auto found_client = find_available_client(browser_dir);
                    if (!found_client.found) {
                        return -1;
                    }
                    auto client_dll = native_library(found_client.dll_path.c_str());
                    if (auto fn = client_dll.get(
                        webview2_symbols::CreateWebViewEnvironmentWithOptionsInternal)) {
                        return fn(true, found_client.runtime_type, user_data_dir, env_options,
                            created_handler);
                    }
                    if (auto fn = client_dll.get(webview2_symbols::DllCanUnloadNow)) {
                        if (!fn()) {
                            client_dll.detach();
                        }
                    }
                    return ERROR_SUCCESS;
                }

                HRESULT
                    get_available_browser_version_string_impl(PCWSTR browser_dir,
                        LPWSTR* version) const {
                    if (!version) {
                        return -1;
                    }
                    auto found_client = find_available_client(browser_dir);
                    if (!found_client.found) {
                        return -1;
                    }
                    auto info_length_bytes =
                        found_client.version.size() * sizeof(found_client.version[0]);
                    auto info = static_cast<LPWSTR>(CoTaskMemAlloc(info_length_bytes));
                    if (!info) {
                        return -1;
                    }
                    CopyMemory(info, found_client.version.c_str(), info_length_bytes);
                    *version = info;
                    return 0;
                }

                client_info_t find_available_client(PCWSTR browser_dir) const {
                    if (browser_dir) {
                        return find_embedded_client(api_version, browser_dir);
                    }
                    auto found_client =
                        find_installed_client(api_version, true, default_release_channel_guid);
                    if (!found_client.found) {
                        found_client = find_installed_client(api_version, false,
                            default_release_channel_guid);
                    }
                    return found_client;
                }

                std::wstring make_client_dll_path(const std::wstring& dir) const {
                    auto dll_path = dir;
                    if (!dll_path.empty()) {
                        auto last_char = dir[dir.size() - 1];
                        if (last_char != L'\\' && last_char != L'/') {
                            dll_path += L'\\';
                        }
                    }
                    dll_path += L"EBWebView\\";
#if defined(_M_X64) || defined(__x86_64__)
                    dll_path += L"x64";
#elif defined(_M_IX86) || defined(__i386__)
                    dll_path += L"x86";
#elif defined(_M_ARM64) || defined(__aarch64__)
                    dll_path += L"arm64";
#else
#error WebView2 integration for this platform is not yet supported.
#endif
                    dll_path += L"\\EmbeddedBrowserWebView.dll";
                    return dll_path;
                }

                client_info_t
                    find_installed_client(unsigned int min_api_version, bool system,
                        const std::wstring& release_channel) const {
                    std::wstring sub_key = client_state_reg_sub_key;
                    sub_key += release_channel;
                    auto root_key = system ? HKEY_LOCAL_MACHINE : HKEY_CURRENT_USER;
                    reg_key key(root_key, sub_key, 0, KEY_READ | KEY_WOW64_32KEY);
                    if (!key.is_open()) {
                        return {};
                    }
                    auto ebwebview_value = key.query_string(L"EBWebView");

                    auto client_version_string =
                        get_last_native_path_component(ebwebview_value);
                    auto client_version = parse_version(client_version_string);
                    if (client_version[2] < min_api_version) {
                        // Our API version is greater than the runtime API version.
                        return {};
                    }

                    auto client_dll_path = make_client_dll_path(ebwebview_value);
                    return { true, client_dll_path, client_version_string,
                            webview2_runtime_type::installed };
                }

                client_info_t find_embedded_client(unsigned int min_api_version,
                    const std::wstring& dir) const {
                    auto client_dll_path = make_client_dll_path(dir);

                    auto client_version_string = get_file_version_string(client_dll_path);
                    auto client_version = parse_version(client_version_string);
                    if (client_version[2] < min_api_version) {
                        // Our API version is greater than the runtime API version.
                        return {};
                    }

                    return { true, client_dll_path, client_version_string,
                            webview2_runtime_type::embedded };
                }

                // The minimum WebView2 API version we need regardless of the SDK release
                // actually used. The number comes from the SDK release version,
                // e.g. 1.0.1150.38. To be safe the SDK should have a number that is greater
                // than or equal to this number. The Edge browser webview client must
                // have a number greater than or equal to this number.
                static constexpr unsigned int api_version = 1150;

                static constexpr auto client_state_reg_sub_key =
                    L"SOFTWARE\\Microsoft\\EdgeUpdate\\ClientState\\";

                // GUID for the stable release channel.
                static constexpr auto stable_release_guid =
                    L"{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

                static constexpr auto default_release_channel_guid = stable_release_guid;
#endif /* WEBVIEW_MSWEBVIEW2_BUILTIN_IMPL */

#if WEBVIEW_MSWEBVIEW2_EXPLICIT_LINK == 1
                native_library m_lib{ L"WebView2Loader.dll" };
#endif
            };

            namespace cast_info {
                static constexpr auto controller_completed =
                    cast_info_t<ICoreWebView2CreateCoreWebView2ControllerCompletedHandler>{
                        IID_ICoreWebView2CreateCoreWebView2ControllerCompletedHandler };

                static constexpr auto environment_completed =
                    cast_info_t<ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler>{
                        IID_ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler };

                static constexpr auto message_received =
                    cast_info_t<ICoreWebView2WebMessageReceivedEventHandler>{
                        IID_ICoreWebView2WebMessageReceivedEventHandler };

                static constexpr auto permission_requested =
                    cast_info_t<ICoreWebView2PermissionRequestedEventHandler>{
                        IID_ICoreWebView2PermissionRequestedEventHandler };
            } // namespace cast_info
        } // namespace mswebview2

        class webview2_com_handler
            : public ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
            public ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
            public ICoreWebView2WebMessageReceivedEventHandler,
            public ICoreWebView2PermissionRequestedEventHandler {
            using webview2_com_handler_cb_t =
                std::function<void(ICoreWebView2Controller*, ICoreWebView2* webview)>;

        public:
            webview2_com_handler(HWND hwnd, msg_cb_t msgCb, webview2_com_handler_cb_t cb)
                : m_window(hwnd), m_msgCb(msgCb), m_cb(cb) {
            }

            virtual ~webview2_com_handler() = default;
            webview2_com_handler(const webview2_com_handler& other) = delete;
            webview2_com_handler& operator=(const webview2_com_handler& other) = delete;
            webview2_com_handler(webview2_com_handler&& other) = delete;
            webview2_com_handler& operator=(webview2_com_handler&& other) = delete;

            ULONG STDMETHODCALLTYPE AddRef() { return ++m_ref_count; }
            ULONG STDMETHODCALLTYPE Release() {
                if (m_ref_count > 1) {
                    return --m_ref_count;
                }
                delete this;
                return 0;
            }
            HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, LPVOID* ppv) {
                using namespace mswebview2::cast_info;

                if (!ppv) {
                    return E_POINTER;
                }

                // All of the COM interfaces we implement should be added here regardless
                // of whether they are required.
                // This is just to be on the safe side in case the WebView2 Runtime ever
                // requests a pointer to an interface we implement.
                // The WebView2 Runtime must at the very least be able to get a pointer to
                // ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler when we use
                // our custom WebView2 loader implementation, and observations have shown
                // that it is the only interface requested in this case. None have been
                // observed to be requested when using the official WebView2 loader.

                if (cast_if_equal_iid(riid, controller_completed, ppv) ||
                    cast_if_equal_iid(riid, environment_completed, ppv) ||
                    cast_if_equal_iid(riid, message_received, ppv) ||
                    cast_if_equal_iid(riid, permission_requested, ppv)) {
                    return S_OK;
                }

                return E_NOINTERFACE;
            }
            HRESULT STDMETHODCALLTYPE Invoke(HRESULT res, ICoreWebView2Environment* env) {
                if (SUCCEEDED(res)) {
                    res = env->CreateCoreWebView2Controller(m_window, this);
                    if (SUCCEEDED(res)) {
                        return S_OK;
                    }
                }
                try_create_environment();
                return S_OK;
            }
            HRESULT STDMETHODCALLTYPE Invoke(HRESULT res,
                ICoreWebView2Controller* controller) {
                if (FAILED(res)) {
                    // See try_create_environment() regarding
                    // HRESULT_FROM_WIN32(ERROR_INVALID_STATE).
                    // The result is E_ABORT if the parent window has been destroyed already.
                    switch (res) {
                    case HRESULT_FROM_WIN32(ERROR_INVALID_STATE):
                    case E_ABORT:
                        return S_OK;
                    }
                    try_create_environment();
                    return S_OK;
                }

                ICoreWebView2* webview;
                ::EventRegistrationToken token;
                controller->get_CoreWebView2(&webview);
                webview->add_WebMessageReceived(this, &token);
                webview->add_PermissionRequested(this, &token);

                m_cb(controller, webview);
                return S_OK;
            }
            HRESULT STDMETHODCALLTYPE Invoke(
                ICoreWebView2* sender, ICoreWebView2WebMessageReceivedEventArgs* args) {
                LPWSTR message;
                args->TryGetWebMessageAsString(&message);
                m_msgCb(narrow_string(message));
                sender->PostWebMessageAsString(message);

                CoTaskMemFree(message);
                return S_OK;
            }
            HRESULT STDMETHODCALLTYPE Invoke(
                ICoreWebView2* sender, ICoreWebView2PermissionRequestedEventArgs* args) {
                (void)sender; // intentionally unused, was used for something back then. I'll probably remove it later on in life. - pparaxan
                COREWEBVIEW2_PERMISSION_KIND kind;
                args->get_PermissionKind(&kind);
                if (kind == COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ) {
                    args->put_State(COREWEBVIEW2_PERMISSION_STATE_ALLOW);
                }
                return S_OK;
            }

            // Checks whether the specified IID equals the IID of the specified type and
            // if so casts the "this" pointer to T and returns it. Returns nullptr on
            // mismatching IIDs.
            // If ppv is specified then the pointer will also be assigned to *ppv.
            template <typename T>
            T* cast_if_equal_iid(REFIID riid, const cast_info_t<T>& info,
                LPVOID* ppv = nullptr) noexcept {
                T* ptr = nullptr;
                if (IsEqualIID(riid, info.iid)) {
                    ptr = static_cast<T*>(this);
                    ptr->AddRef();
                }
                if (ppv) {
                    *ppv = ptr;
                }
                return ptr;
            }

            // Set the function that will perform the initiating logic for creating
            // the WebView2 environment.
            void set_attempt_handler(std::function<HRESULT()> attempt_handler) noexcept {
                m_attempt_handler = attempt_handler;
            }

            // Retry creating a WebView2 environment.
            // The initiating logic for creating the environment is defined by the
            // caller of set_attempt_handler().
            void try_create_environment() noexcept {
                // WebView creation fails with HRESULT_FROM_WIN32(ERROR_INVALID_STATE) if
                // a running instance using the same user data folder exists, and the
                // Environment objects have different EnvironmentOptions.
                // Source: https://docs.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment?view=webview2-1.0.1150.38
                if (m_attempts < m_max_attempts) {
                    ++m_attempts;
                    auto res = m_attempt_handler();
                    if (SUCCEEDED(res)) {
                        return;
                    }
                    // Not entirely sure if this error code only applies to
                    // CreateCoreWebView2Controller so we check here as well.
                    if (res == HRESULT_FROM_WIN32(ERROR_INVALID_STATE)) {
                        return;
                    }
                    try_create_environment();
                    return;
                }
                // Give up.
                m_cb(nullptr, nullptr);
            }

        private:
            HWND m_window;
            msg_cb_t m_msgCb;
            webview2_com_handler_cb_t m_cb;
            std::atomic<ULONG> m_ref_count{ 1 };
            std::function<HRESULT()> m_attempt_handler;
            unsigned int m_max_attempts = 5;
            unsigned int m_attempts = 0;
        };

        class win32_edge_engine {
        public:
            win32_edge_engine(bool debug, void* window) {
                if (!is_webview2_available()) {
                    return;
                }
                if (!m_com_init.is_initialized()) {
                    return;
                }
                enable_dpi_awareness();
                if (window == nullptr) {
                    HINSTANCE hInstance = GetModuleHandle(nullptr);
                    HICON icon = (HICON)LoadImage(
                        hInstance, IDI_APPLICATION, IMAGE_ICON, GetSystemMetrics(SM_CXICON),
                        GetSystemMetrics(SM_CYICON), LR_DEFAULTCOLOR);

                    WNDCLASSEXW wc;
                    ZeroMemory(&wc, sizeof(WNDCLASSEX));
                    wc.cbSize = sizeof(WNDCLASSEX);
                    wc.hInstance = hInstance;
                    wc.lpszClassName = L"webview";
                    wc.hIcon = icon;
                    wc.lpfnWndProc =
                        (WNDPROC)(+[](HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) -> LRESULT {
                        auto w = (win32_edge_engine*)GetWindowLongPtr(hwnd, GWLP_USERDATA);
                        switch (msg) {
                        case WM_SIZE:
                            w->resize(hwnd);
                            break;
                        case WM_CLOSE:
                            DestroyWindow(hwnd);
                            break;
                        case WM_DESTROY:
                            w->terminate();
                            break;
                        case WM_GETMINMAXINFO: {
                            auto lpmmi = (LPMINMAXINFO)lp;
                            if (w == nullptr) {
                                return 0;
                            }
                            if (w->m_maxsz.x > 0 && w->m_maxsz.y > 0) {
                                lpmmi->ptMaxSize = w->m_maxsz;
                                lpmmi->ptMaxTrackSize = w->m_maxsz;
                            }
                            if (w->m_minsz.x > 0 && w->m_minsz.y > 0) {
                                lpmmi->ptMinTrackSize = w->m_minsz;
                            }
                        } break;
                        default:
                            return DefWindowProcW(hwnd, msg, wp, lp);
                        }
                        return 0;
                            });
                    RegisterClassExW(&wc);
                    m_window = CreateWindowW(L"webview", L"", WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT, CW_USEDEFAULT, 640, 480, nullptr,
                        nullptr, hInstance, nullptr);
                    if (m_window == nullptr) {
                        return;
                    }
                    SetWindowLongPtr(m_window, GWLP_USERDATA, (LONG_PTR)this);
                }
                else {
                    m_window = *(static_cast<HWND*>(window));
                }

                ShowWindow(m_window, SW_SHOW);
                UpdateWindow(m_window);
                SetFocus(m_window);

                auto cb =
                    std::bind(&win32_edge_engine::on_message, this, std::placeholders::_1);

                embed(m_window, debug, cb);
                resize(m_window);
                m_controller->MoveFocus(COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
            }

            virtual ~win32_edge_engine() {
                if (m_com_handler) {
                    m_com_handler->Release();
                    m_com_handler = nullptr;
                }
                if (m_webview) {
                    m_webview->Release();
                    m_webview = nullptr;
                }
                if (m_controller) {
                    m_controller->Release();
                    m_controller = nullptr;
                }
            }

            win32_edge_engine(const win32_edge_engine& other) = delete;
            win32_edge_engine& operator=(const win32_edge_engine& other) = delete;
            win32_edge_engine(win32_edge_engine&& other) = delete;
            win32_edge_engine& operator=(win32_edge_engine&& other) = delete;

            void run() {
                MSG msg;
                BOOL res;
                while ((res = GetMessage(&msg, nullptr, 0, 0)) != -1) {
                    if (msg.hwnd) {
                        TranslateMessage(&msg);
                        DispatchMessage(&msg);
                        continue;
                    }
                    if (msg.message == WM_APP) {
                        auto f = (dispatch_fn_t*)(msg.lParam);
                        (*f)();
                        delete f;
                    }
                    else if (msg.message == WM_QUIT) {
                        return;
                    }
                }
            }

            void* window() { return (void*)m_window; }

            void terminate() { PostQuitMessage(0); }

            void dispatch(dispatch_fn_t f) {
                PostThreadMessage(m_main_thread, WM_APP, 0, (LPARAM) new dispatch_fn_t(f));
            }

            void set_title(const std::string& title) {
                SetWindowTextW(m_window, widen_string(title).c_str());
            }

            void set_size(int width, int height, int hints) {
                auto style = GetWindowLong(m_window, GWL_STYLE);
                if (hints == WEBVIEW_HINT_FIXED) {
                    style &= ~(WS_THICKFRAME | WS_MAXIMIZEBOX);
                }
                else {
                    style |= (WS_THICKFRAME | WS_MAXIMIZEBOX);
                }
                SetWindowLong(m_window, GWL_STYLE, style);

                if (hints == WEBVIEW_HINT_MAX) {
                    m_maxsz.x = width;
                    m_maxsz.y = height;
                }
                else if (hints == WEBVIEW_HINT_MIN) {
                    m_minsz.x = width;
                    m_minsz.y = height;
                }
                else {
                    RECT r;
                    r.left = r.top = 0;
                    r.right = width;
                    r.bottom = height;
                    AdjustWindowRect(&r, WS_OVERLAPPEDWINDOW, 0);
                    SetWindowPos(
                        m_window, nullptr, r.left, r.top, r.right - r.left, r.bottom - r.top,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE | SWP_FRAMECHANGED);
                    resize(m_window);
                }
            }

            void navigate(const std::string& url) {
                auto wurl = widen_string(url);
                m_webview->Navigate(wurl.c_str());
            }

            void init(const std::string& js) {
                auto wjs = widen_string(js);
                m_webview->AddScriptToExecuteOnDocumentCreated(wjs.c_str(), nullptr);
            }

            void eval(const std::string& js) {
                auto wjs = widen_string(js);
                m_webview->ExecuteScript(wjs.c_str(), nullptr);
            }

            void set_html(const std::string& html) {
                m_webview->NavigateToString(widen_string(html).c_str());
            }

        private:
            bool embed(HWND wnd, bool debug, msg_cb_t cb) {
                std::atomic_flag flag = ATOMIC_FLAG_INIT;
                flag.test_and_set();

                wchar_t currentExePath[MAX_PATH];
                GetModuleFileNameW(nullptr, currentExePath, MAX_PATH);
                wchar_t* currentExeName = PathFindFileNameW(currentExePath);

                wchar_t dataPath[MAX_PATH];
                if (!SUCCEEDED(
                    SHGetFolderPathW(nullptr, CSIDL_APPDATA, nullptr, 0, dataPath))) {
                    return false;
                }
                wchar_t userDataFolder[MAX_PATH];
                PathCombineW(userDataFolder, dataPath, currentExeName);

                m_com_handler = new webview2_com_handler(
                    wnd, cb,
                    [&](ICoreWebView2Controller* controller, ICoreWebView2* webview) {
                        if (!controller || !webview) {
                            flag.clear();
                            return;
                        }
                        controller->AddRef();
                        webview->AddRef();
                        m_controller = controller;
                        m_webview = webview;
                        flag.clear();
                    });

                m_com_handler->set_attempt_handler([&] {
                    return m_webview2_loader.create_environment_with_options(
                        nullptr, userDataFolder, nullptr, m_com_handler);
                    });
                m_com_handler->try_create_environment();

                MSG msg = {};
                while (flag.test_and_set() && GetMessage(&msg, nullptr, 0, 0)) {
                    TranslateMessage(&msg);
                    DispatchMessage(&msg);
                }
                if (!m_controller || !m_webview) {
                    return false;
                }
                ICoreWebView2Settings* settings = nullptr;
                auto res = m_webview->get_Settings(&settings);
                if (res != S_OK) {
                    return false;
                }
                res = settings->put_AreDevToolsEnabled(debug ? TRUE : FALSE);
                if (res != S_OK) {
                    return false;
                }
                init("window.external={invoke:s=>window.chrome.webview.postMessage(s)}");
                return true;
            }

            void resize(HWND wnd) {
                if (m_controller == nullptr) {
                    return;
                }
                RECT bounds;
                GetClientRect(wnd, &bounds);
                m_controller->put_Bounds(bounds);
            }

            bool is_webview2_available() const noexcept {
                LPWSTR version_info = nullptr;
                auto res = m_webview2_loader.get_available_browser_version_string(
                    nullptr, &version_info);
                // The result will be equal to HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND)
                // if the WebView2 runtime is not installed.
                auto ok = SUCCEEDED(res) && version_info;
                if (version_info) {
                    CoTaskMemFree(version_info);
                }
                return ok;
            }

            virtual void on_message(const std::string& msg) = 0;

            // The app is expected to call CoInitializeEx before
            // CreateCoreWebView2EnvironmentWithOptions.
            // Source: https://docs.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl#createcorewebview2environmentwithoptions
            com_init_wrapper m_com_init{ COINIT_APARTMENTTHREADED };
            HWND m_window = nullptr;
            POINT m_minsz = POINT{ 0, 0 };
            POINT m_maxsz = POINT{ 0, 0 };
            DWORD m_main_thread = GetCurrentThreadId();
            ICoreWebView2* m_webview = nullptr;
            ICoreWebView2Controller* m_controller = nullptr;
            webview2_com_handler* m_com_handler = nullptr;
            mswebview2::loader m_webview2_loader;
        };

    } // namespace detail

    using browser_engine = detail::win32_edge_engine;

    class webview : public browser_engine {
    public:
        webview(bool debug = false, void* wnd = nullptr)
            : browser_engine(debug, wnd) {
        }

        void navigate(const std::string& url) {
            if (url.empty()) {
                browser_engine::navigate("about:blank");
                return;
            }
            browser_engine::navigate(url);
        }

        using binding_t = std::function<void(std::string, std::string, void*)>;
        class binding_ctx_t {
        public:
            binding_ctx_t(binding_t callback, void* arg)
                : callback(callback), arg(arg) {
            }
            // This function is called upon execution of the bound JS function
            binding_t callback;
            // This user-supplied argument is passed to the callback
            void* arg;
        };

        using sync_binding_t = std::function<std::string(std::string)>;

        // Synchronous bind
        void bind(const std::string& name, sync_binding_t fn) {
            auto wrapper = [this, fn](const std::string& seq, const std::string& req,
                void* /*arg*/) { resolve(seq, 0, fn(req)); };
            bind(name, wrapper, nullptr);
        }

        // Asynchronous bind
        void bind(const std::string& name, binding_t fn, void* arg) {
            if (bindings.count(name) > 0) {
                return;
            }
            bindings.emplace(name, binding_ctx_t(fn, arg));
            auto js = "(function() { var name = '" + name + "';" + R""(
      var RPC = window._rpc = (window._rpc || {nextSeq: 1});
      window[name] = function() {
        var seq = RPC.nextSeq++;
        var promise = new Promise(function(resolve, reject) {
          RPC[seq] = {
            resolve: resolve,
            reject: reject,
          };
        });
        window.external.invoke(JSON.stringify({
          id: seq,
          method: name,
          params: Array.prototype.slice.call(arguments),
        }));
        return promise;
      }
    })())"";
            init(js);
            eval(js);
        }

        void unbind(const std::string& name) {
            auto found = bindings.find(name);
            if (found != bindings.end()) {
                auto js = "delete window['" + name + "'];";
                init(js);
                eval(js);
                bindings.erase(found);
            }
        }

        void resolve(const std::string& seq, int status, const std::string& result) {
            dispatch([seq, status, result, this]() {
                if (status == 0) {
                    eval("window._rpc[" + seq + "].resolve(" + result +
                        "); delete window._rpc[" + seq + "]");
                }
                else {
                    eval("window._rpc[" + seq + "].reject(" + result +
                        "); delete window._rpc[" + seq + "]");
                }
                });
        }

    private:
        void on_message(const std::string& msg) {
            auto seq = detail::json_parse(msg, "id", 0);
            auto name = detail::json_parse(msg, "method", 0);
            auto args = detail::json_parse(msg, "params", 0);
            auto found = bindings.find(name);
            if (found == bindings.end()) {
                return;
            }
            const auto& context = found->second;
            context.callback(seq, args, context.arg);
        }

        std::map<std::string, binding_ctx_t> bindings;
    };

} // namespace webview

#endif
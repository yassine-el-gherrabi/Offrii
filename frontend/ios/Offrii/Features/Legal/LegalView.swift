import SwiftUI
import WebKit

// MARK: - LegalView

struct LegalView: View {
    enum Page {
        case mentions
        case privacy
        case terms
    }

    let page: Page

    private var pageURL: URL {
        let lang = Locale.current.language.languageCode?.identifier ?? "fr"
        let base = APIEndpoint.baseURL
        switch page {
        case .mentions: return URL(string: "\(base)/legal/mentions?lang=\(lang)")!
        case .privacy:  return URL(string: "\(base)/legal/privacy?lang=\(lang)")!
        case .terms:    return URL(string: "\(base)/legal/terms?lang=\(lang)")!
        }
    }

    private var pageTitle: String {
        switch page {
        case .mentions: return NSLocalizedString("profile.legal", comment: "")
        case .privacy:  return NSLocalizedString("profile.privacy", comment: "")
        case .terms:    return NSLocalizedString("profile.terms", comment: "")
        }
    }

    var body: some View {
        LegalWebView(url: pageURL)
            .navigationTitle(pageTitle)
            .navigationBarTitleDisplayMode(.inline)
            .background(OffriiTheme.surface.ignoresSafeArea())
    }
}

// MARK: - WebView

struct LegalWebView: UIViewRepresentable {
    let url: URL

    func makeUIView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        let webView = WKWebView(frame: .zero, configuration: config)
        webView.isOpaque = false
        webView.backgroundColor = .clear
        webView.scrollView.backgroundColor = .clear
        webView.load(URLRequest(url: url))
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {}
}

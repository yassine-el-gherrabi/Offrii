import SwiftUI

struct SplashView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(AppRouter.self) private var router

    @State private var logoVisible = false

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            BlobBackground(preset: .auth)
                .ignoresSafeArea()
                .opacity(0.5)

            VStack(spacing: OffriiTheme.spacingMD) {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG)
                    .fill(OffriiTheme.primary)
                    .frame(width: 88, height: 88)
                    .overlay(
                        Image(systemName: "gift.fill")
                            .font(.system(size: 40))
                            .foregroundColor(.white)
                    )
                    .scaleEffect(logoVisible ? 1.0 : 0.8)
                    .opacity(logoVisible ? 1.0 : 0)

                Text("Offrii")
                    .font(OffriiTypography.displayLarge)
                    .foregroundColor(OffriiTheme.text)
                    .opacity(logoVisible ? 1.0 : 0)

                Text(NSLocalizedString("splash.tagline", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textSecondary)
                    .opacity(logoVisible ? 1.0 : 0)
            }
        }
        .task {
            withAnimation(OffriiAnimation.bouncy) {
                logoVisible = true
            }

            async let authResult: Bool = authManager.refreshAndLoadUser()
            async let minimumDelay: Void = Task.sleep(for: .milliseconds(800))

            let isAuthenticated = await authResult
            _ = try? await minimumDelay

            withAnimation(OffriiAnimation.modal) {
                router.resolvePostSplash(isAuthenticated: isAuthenticated)
            }
        }
    }
}

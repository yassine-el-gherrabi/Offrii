import SwiftUI

struct OnboardingView: View {
    @State private var currentPage = 0
    let onComplete: () -> Void
    let onSignIn: () -> Void

    private let steps = OnboardingStep.steps

    var body: some View {
        ZStack {
            OffriiTheme.primary.ignoresSafeArea()
            DecorativeSquares(preset: .fullScreen)

            VStack(spacing: 0) {
                // Skip button
                HStack {
                    Spacer()
                    Button(NSLocalizedString("onboarding.skip", comment: "")) {
                        onComplete()
                    }
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(.white.opacity(0.8))
                    .padding(.trailing, OffriiTheme.spacingLG)
                    .padding(.top, OffriiTheme.spacingMD)
                }

                Spacer()

                // Content
                TabView(selection: $currentPage) {
                    ForEach(steps) { step in
                        VStack(spacing: OffriiTheme.spacingLG) {
                            Text(step.emoji)
                                .font(.system(size: 80))

                            Text(NSLocalizedString(step.titleKey, comment: ""))
                                .font(OffriiTypography.title)
                                .foregroundColor(.white)
                                .multilineTextAlignment(.center)

                            Text(NSLocalizedString(step.subtitleKey, comment: ""))
                                .font(OffriiTypography.body)
                                .foregroundColor(.white.opacity(0.8))
                                .multilineTextAlignment(.center)
                                .padding(.horizontal, OffriiTheme.spacingXL)
                        }
                        .tag(step.id)
                    }
                }
                .tabViewStyle(.page(indexDisplayMode: .never))

                Spacer()

                // Dots
                HStack(spacing: OffriiTheme.spacingSM) {
                    ForEach(steps) { step in
                        Circle()
                            .fill(currentPage == step.id ? Color.white : Color.white.opacity(0.4))
                            .frame(width: 8, height: 8)
                            .animation(OffriiTheme.defaultAnimation, value: currentPage)
                    }
                }
                .padding(.bottom, OffriiTheme.spacingLG)

                // CTA
                VStack(spacing: OffriiTheme.spacingMD) {
                    Button {
                        if currentPage < steps.count - 1 {
                            withAnimation { currentPage += 1 }
                        } else {
                            onComplete()
                        }
                    } label: {
                        Text(currentPage < steps.count - 1
                             ? NSLocalizedString("onboarding.continue", comment: "")
                             : NSLocalizedString("onboarding.start", comment: ""))
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.primary)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, OffriiTheme.spacingMD)
                        .background(Color.white)
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)

                    if currentPage == steps.count - 1 {
                        Button {
                            onSignIn()
                        } label: {
                            Text(NSLocalizedString("onboarding.alreadyAccount", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(.white.opacity(0.8))
                        }
                    }
                }
                .padding(.bottom, OffriiTheme.spacingXL)
            }
        }
    }
}

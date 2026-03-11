import SwiftUI

// MARK: - Feature Descriptor

private struct FeatureDescriptor {
    let icon: String
    let color: Color
    let titleKey: String
    let subtitleKey: String
}

struct WelcomeView: View {
    @Environment(AppRouter.self) private var router
    @State private var currentPage = 0

    // Amber/golden for Entraide — warmth, generosity
    fileprivate static let amberColor = Color(red: 0.96, green: 0.68, blue: 0.22)

    private static let features: [FeatureDescriptor] = [
        FeatureDescriptor(icon: "heart.text.clipboard", color: OffriiTheme.primary, titleKey: "onboarding.step1.title", subtitleKey: "onboarding.step1.subtitle"),
        FeatureDescriptor(icon: "person.2.circle", color: OffriiTheme.accent, titleKey: "onboarding.step2.title", subtitleKey: "onboarding.step2.subtitle"),
        FeatureDescriptor(icon: "hands.sparkles", color: amberColor, titleKey: "onboarding.step3.title", subtitleKey: "onboarding.step3.subtitle")
    ]

    private static let pageCount = features.count + 1 // 3 features + 1 summary
    private var isLastPage: Bool { currentPage == Self.pageCount - 1 }

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            DiffuseBlobBackground()
                .ignoresSafeArea()

            VStack(spacing: 0) {
                // Skip button — hidden on last page
                HStack {
                    Spacer()
                    if !isLastPage {
                        Button {
                            router.completeOnboarding()
                        } label: {
                            Text(NSLocalizedString("onboarding.skip", comment: ""))
                                .font(OffriiTypography.subheadline)
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                        .transition(.opacity)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingXL)
                .padding(.top, OffriiTheme.spacingLG)
                .frame(height: 44)
                .animation(OffriiAnimation.defaultSpring, value: isLastPage)

                // Paged content
                TabView(selection: $currentPage) {
                    ForEach(Array(Self.features.enumerated()), id: \.offset) { index, feature in
                        featurePage(feature: feature)
                            .tag(index)
                    }

                    SummaryPage(features: Self.features)
                        .tag(Self.pageCount - 1)
                }
                .tabViewStyle(.page(indexDisplayMode: .never))
                .animation(OffriiAnimation.defaultSpring, value: currentPage)

                // Custom pill dots
                PillPageIndicator(
                    currentPage: currentPage,
                    pageCount: Self.pageCount
                )
                .padding(.bottom, OffriiTheme.spacingLG)

                // Bottom CTAs — last page only
                if isLastPage {
                    bottomButtons
                        .padding(.horizontal, OffriiTheme.spacingXL)
                        .padding(.bottom, OffriiTheme.spacingXXL)
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                }
            }
            .animation(OffriiAnimation.modal, value: isLastPage)
        }
    }

    // MARK: - Feature Page (pages 1-3)

    private func featurePage(feature: FeatureDescriptor) -> some View {
        VStack(spacing: OffriiTheme.spacingLG) {
            Spacer()

            ShinyIcon(systemName: feature.icon, color: feature.color)
                .padding(.bottom, OffriiTheme.spacingSM)

            Text(NSLocalizedString(feature.titleKey, comment: ""))
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)

            Text(NSLocalizedString(feature.subtitleKey, comment: ""))
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, OffriiTheme.spacingXL)

            Spacer()
            Spacer()
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
    }

    // MARK: - Bottom Buttons (last page only)

    private var bottomButtons: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            OffriiButton(
                NSLocalizedString("onboarding.start", comment: ""),
                variant: .primary
            ) {
                router.completeOnboarding()
            }

            Button {
                router.completeOnboardingToLogin()
            } label: {
                Text(NSLocalizedString("onboarding.alreadyAccount", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.primary)
                    .underline()
            }
        }
    }
}

// MARK: - Summary Page (page 4 — fanned cards)

private struct SummaryPage: View {
    let features: [FeatureDescriptor]

    @State private var appeared = false

    // Card labels (short, same in both languages for the fan)
    private static let cardLabels = [
        "onboarding.feature.wishes",
        "onboarding.feature.share",
        "onboarding.feature.help"
    ]

    var body: some View {
        VStack(spacing: OffriiTheme.spacingXL) {
            Spacer()

            // Fanned cards
            ZStack {
                // Left card — Partage
                FeatureMiniCard(
                    icon: features[1].icon,
                    color: features[1].color,
                    labelKey: Self.cardLabels[1]
                )
                .rotationEffect(.degrees(-8))
                .offset(x: appeared ? -65 : 0)
                .scaleEffect(appeared ? 0.92 : 0.7)
                .opacity(appeared ? 1 : 0)

                // Right card — Entraide
                FeatureMiniCard(
                    icon: features[2].icon,
                    color: features[2].color,
                    labelKey: Self.cardLabels[2]
                )
                .rotationEffect(.degrees(8))
                .offset(x: appeared ? 65 : 0)
                .scaleEffect(appeared ? 0.92 : 0.7)
                .opacity(appeared ? 1 : 0)

                // Center card — Envies (on top, slightly larger)
                FeatureMiniCard(
                    icon: features[0].icon,
                    color: features[0].color,
                    labelKey: Self.cardLabels[0]
                )
                .scaleEffect(appeared ? 1.0 : 0.7)
                .opacity(appeared ? 1 : 0)
            }
            .frame(height: 170)

            Text(NSLocalizedString("onboarding.summary.title", comment: ""))
                .font(OffriiTypography.titleLarge)
                .foregroundColor(OffriiTheme.text)
                .multilineTextAlignment(.center)
                .opacity(appeared ? 1 : 0)

            Text(NSLocalizedString("onboarding.summary.subtitle", comment: ""))
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, OffriiTheme.spacingXL)
                .opacity(appeared ? 1 : 0)

            Spacer()
            Spacer()
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .onAppear {
            withAnimation(OffriiAnimation.bouncy.delay(0.15)) {
                appeared = true
            }
        }
        .onDisappear { appeared = false }
    }
}

// MARK: - Feature Mini Card

private struct FeatureMiniCard: View {
    let icon: String
    let color: Color
    let labelKey: String

    var body: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            Image(systemName: icon)
                .font(.system(size: 32))
                .foregroundColor(color)

            Text(NSLocalizedString(labelKey, comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.textSecondary)
        }
        .frame(width: 110, height: 130)
        .background(OffriiTheme.card)
        .clipShape(RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusLG, style: .continuous))
        .shadow(color: .black.opacity(0.08), radius: 12, y: 4)
    }
}

// MARK: - Shiny Icon

private struct ShinyIcon: View {
    let systemName: String
    let color: Color

    @State private var shimmer = false

    var body: some View {
        let icon = Image(systemName: systemName)
            .font(.system(size: 60))

        icon
            .foregroundColor(color)
            .overlay {
                Rectangle()
                    .fill(
                        .linearGradient(
                            colors: [.clear, .white.opacity(0.45), .clear],
                            startPoint: .leading,
                            endPoint: .trailing
                        )
                    )
                    .frame(width: 28)
                    .rotationEffect(.degrees(25))
                    .offset(x: shimmer ? 80 : -80)
                    .mask { icon }
            }
            .onAppear {
                withAnimation(
                    .easeInOut(duration: 2.5)
                    .repeatForever(autoreverses: false)
                ) {
                    shimmer = true
                }
            }
    }
}

// MARK: - Diffuse Blob Background

private struct DiffuseBlobBackground: View {
    @State private var drifting = false

    var body: some View {
        ZStack {
            // Coral blob — top right
            AnimatedBlobView(
                color: OffriiTheme.primary.opacity(0.16),
                size: 300
            )
            .offset(
                x: drifting ? 110 : 130,
                y: drifting ? -100 : -70
            )

            // Teal blob — bottom left
            AnimatedBlobView(
                color: OffriiTheme.accent.opacity(0.13),
                size: 260
            )
            .offset(
                x: drifting ? -120 : -90,
                y: drifting ? 180 : 210
            )

            // Amber blob — top left
            AnimatedBlobView(
                color: WelcomeView.amberColor.opacity(0.12),
                size: 220
            )
            .offset(
                x: drifting ? -100 : -130,
                y: drifting ? -200 : -230
            )
        }
        .blur(radius: 55)
        .onAppear {
            withAnimation(
                .easeInOut(duration: 10)
                .repeatForever(autoreverses: true)
            ) {
                drifting = true
            }
        }
    }
}

// MARK: - Pill Page Indicator

private struct PillPageIndicator: View {
    let currentPage: Int
    let pageCount: Int

    private let dotSize: CGFloat = 8
    private let pillWidth: CGFloat = 24
    private let dotSpacing: CGFloat = 8

    var body: some View {
        HStack(spacing: dotSpacing) {
            ForEach(0..<pageCount, id: \.self) { index in
                Capsule()
                    .fill(index == currentPage ? OffriiTheme.primary : OffriiTheme.border)
                    .frame(
                        width: index == currentPage ? pillWidth : dotSize,
                        height: dotSize
                    )
            }
        }
        .animation(OffriiAnimation.snappy, value: currentPage)
    }
}

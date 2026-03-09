import Foundation

struct OnboardingStep: Identifiable {
    let id: Int
    let emoji: String
    let titleKey: String
    let subtitleKey: String

    static let steps: [OnboardingStep] = [
        OnboardingStep(
            id: 0,
            emoji: "🎁",
            titleKey: "onboarding.step1.title",
            subtitleKey: "onboarding.step1.subtitle"
        ),
        OnboardingStep(
            id: 1,
            emoji: "👥",
            titleKey: "onboarding.step2.title",
            subtitleKey: "onboarding.step2.subtitle"
        ),
        OnboardingStep(
            id: 2,
            emoji: "🤲",
            titleKey: "onboarding.step3.title",
            subtitleKey: "onboarding.step3.subtitle"
        ),
    ]
}

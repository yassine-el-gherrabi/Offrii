import SwiftUI

// MARK: - ProfileProgressCard

struct ProfileProgressCard: View {
    let progress: ProfileProgress
    @State private var showDetail = false

    var body: some View {
        if progress.percentage >= 100 { EmptyView() } else {
            Button {
                showDetail = true
            } label: {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                    HStack {
                        Text(String(format: NSLocalizedString("progress.title", comment: ""), progress.percentage))
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        Spacer()

                        HStack(spacing: OffriiTheme.spacingXS) {
                            Text("\(progress.completedCount)/\(progress.totalCount)")
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)

                            Image(systemName: "chevron.right")
                                .font(.system(size: 10, weight: .semibold))
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                    }

                    // Progress bar
                    GeometryReader { geometry in
                        ZStack(alignment: .leading) {
                            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusFull)
                                .fill(OffriiTheme.border)
                                .frame(height: 8)

                            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusFull)
                                .fill(
                                    LinearGradient(
                                        colors: [OffriiTheme.primary, OffriiTheme.primaryLight],
                                        startPoint: .leading,
                                        endPoint: .trailing
                                    )
                                )
                                .frame(
                                    width: geometry.size.width * CGFloat(progress.percentage) / 100,
                                    height: 8
                                )
                                .animation(OffriiAnimation.defaultSpring, value: progress.percentage)
                        }
                    }
                    .frame(height: 8)

                    // Next action hint
                    if let nextStep = progress.nextIncompleteStep {
                        HStack(spacing: 6) {
                            Image(systemName: nextStep.icon)
                                .font(.system(size: 11))
                                .foregroundColor(OffriiTheme.primary)
                            Text(String(
                                format: NSLocalizedString("progress.nextStep", comment: ""),
                                NSLocalizedString(nextStep.titleKey, comment: "")
                            ))
                            .font(OffriiTypography.caption)
                            .foregroundColor(OffriiTheme.textSecondary)
                        }
                    }
                }
                .padding(OffriiTheme.spacingBase)
                .background(OffriiTheme.card)
                .cornerRadius(OffriiTheme.cornerRadiusLG)
                .shadow(
                    color: OffriiTheme.cardShadowColor,
                    radius: OffriiTheme.cardShadowRadius,
                    x: 0,
                    y: OffriiTheme.cardShadowY
                )
            }
            .buttonStyle(.plain)
            .sheet(isPresented: $showDetail) {
                ProfileProgressSheet(progress: progress)
                    .presentationDetents([.large])
            }
        }
    }
}

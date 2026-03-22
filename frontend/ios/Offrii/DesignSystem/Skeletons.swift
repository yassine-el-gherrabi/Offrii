import SwiftUI

// MARK: - Shimmer Effect Modifier

struct ShimmerModifier: ViewModifier {
    @State private var phase: CGFloat = -1.0

    func body(content: Content) -> some View {
        content
            .redacted(reason: .placeholder)
            .overlay(
                GeometryReader { geometry in
                    LinearGradient(
                        colors: [
                            .clear,
                            Color.white.opacity(0.4),
                            .clear,
                        ],
                        startPoint: .leading,
                        endPoint: .trailing
                    )
                    .frame(width: geometry.size.width * 0.6)
                    .offset(x: phase * geometry.size.width * 1.6 - geometry.size.width * 0.3)
                }
            )
            .clipped()
            .onAppear {
                withAnimation(
                    .linear(duration: 1.5)
                    .repeatForever(autoreverses: false)
                ) {
                    phase = 1.0
                }
            }
    }
}

extension View {
    func shimmer() -> some View {
        modifier(ShimmerModifier())
    }
}

// MARK: - Skeleton Row

struct SkeletonRow: View {
    var height: CGFloat = 72

    var body: some View {
        HStack(spacing: OffriiTheme.spacingMD) {
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                .fill(OffriiTheme.border.opacity(0.3))
                .frame(width: 44, height: 44)

            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.3))
                    .frame(height: 14)
                    .frame(maxWidth: .infinity)

                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.2))
                    .frame(height: 12)
                    .frame(width: 120)
            }
        }
        .frame(height: height)
        .padding(.horizontal, OffriiTheme.spacingBase)
        .shimmer()
    }
}

// MARK: - Skeleton Card

struct SkeletonCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingMD) {
            HStack {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.3))
                    .frame(width: 60, height: 20)

                Spacer()

                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.2))
                    .frame(width: 50, height: 20)
            }

            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                .fill(OffriiTheme.border.opacity(0.3))
                .frame(height: 16)

            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                .fill(OffriiTheme.border.opacity(0.2))
                .frame(height: 14)
                .frame(width: 200)

            HStack {
                Circle()
                    .fill(OffriiTheme.border.opacity(0.3))
                    .frame(width: 28, height: 28)

                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.2))
                    .frame(width: 80, height: 12)

                Spacer()

                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXS)
                    .fill(OffriiTheme.border.opacity(0.2))
                    .frame(width: 40, height: 12)
            }
        }
        .padding(OffriiTheme.spacingBase)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shimmer()
    }
}

// MARK: - Skeleton List

struct SkeletonList: View {
    var count: Int = 5

    var body: some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            ForEach(0..<count, id: \.self) { _ in
                SkeletonRow()
            }
        }
    }
}

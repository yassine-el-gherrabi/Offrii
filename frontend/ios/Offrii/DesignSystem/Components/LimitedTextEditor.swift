import SwiftUI

struct LimitedTextEditor: View {
    let placeholder: String
    @Binding var text: String
    var maxLength: Int = 500
    var lineLimit: ClosedRange<Int> = 2...4

    private var isOverLimit: Bool {
        text.count > maxLength
    }

    private var showCounter: Bool {
        text.count > maxLength * 4 / 5
    }

    var body: some View {
        VStack(alignment: .trailing, spacing: 4) {
            TextField(placeholder, text: $text, axis: .vertical)
                .font(OffriiTypography.body)
                .lineLimit(lineLimit)
                .padding(OffriiTheme.spacingSM)
                .background(OffriiTheme.surface)
                .cornerRadius(OffriiTheme.cornerRadiusMD)
                .overlay(
                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                        .stroke(isOverLimit ? OffriiTheme.danger : OffriiTheme.border, lineWidth: 1)
                )
                .onChange(of: text) { _, newValue in
                    if newValue.count > maxLength {
                        text = String(newValue.prefix(maxLength))
                    }
                }

            if showCounter {
                Text("\(text.count)/\(maxLength)")
                    .font(.system(size: 11))
                    .foregroundColor(isOverLimit ? OffriiTheme.danger : OffriiTheme.textMuted)
            }
        }
    }
}

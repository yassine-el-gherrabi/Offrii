import SwiftUI
import UIKit

struct OTPField: View {
    @Binding var code: String
    var errorMessage: String?

    private let digitCount = 6

    var body: some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            ZStack {
                // Hidden UITextField that captures real keyboard input
                HiddenTextField(text: $code, digitCount: digitCount)
                    .frame(width: 1, height: 1)
                    .opacity(0.01)

                // Visual digit boxes
                HStack(spacing: OffriiTheme.spacingSM) {
                    // First group of 3
                    HStack(spacing: OffriiTheme.spacingXS) {
                        ForEach(0..<3, id: \.self) { index in
                            digitBox(at: index)
                        }
                    }

                    Text("-")
                        .font(.system(size: 20, weight: .medium, design: .monospaced))
                        .foregroundColor(OffriiTheme.textMuted)

                    // Second group of 3
                    HStack(spacing: OffriiTheme.spacingXS) {
                        ForEach(3..<6, id: \.self) { index in
                            digitBox(at: index)
                        }
                    }
                }
            }

            if let errorMessage {
                Text(errorMessage)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.danger)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func digitBox(at index: Int) -> some View {
        let isActive = code.count == index
        let hasError = errorMessage != nil
        let char = characterAt(index)

        return Text(char)
            .font(.system(size: 24, weight: .bold, design: .monospaced))
            .foregroundColor(OffriiTheme.text)
            .frame(width: 48, height: 48)
            .background(OffriiTheme.surface)
            .clipShape(RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD, style: .continuous)
                    .strokeBorder(
                        hasError ? OffriiTheme.danger : (isActive ? OffriiTheme.primary : OffriiTheme.border),
                        lineWidth: isActive ? 2 : 1
                    )
            )
    }

    private func characterAt(_ index: Int) -> String {
        guard index < code.count else { return "" }
        return String(code[code.index(code.startIndex, offsetBy: index)])
    }
}

// MARK: - Hidden UITextField (UIViewRepresentable)

private struct HiddenTextField: UIViewRepresentable {
    @Binding var text: String
    let digitCount: Int

    func makeUIView(context: Context) -> UITextField {
        let textField = UITextField()
        textField.keyboardType = .numberPad
        textField.textContentType = .oneTimeCode
        textField.delegate = context.coordinator
        textField.addTarget(context.coordinator, action: #selector(Coordinator.textChanged(_:)), for: .editingChanged)
        // Auto-focus
        DispatchQueue.main.async {
            textField.becomeFirstResponder()
        }
        return textField
    }

    func updateUIView(_ uiView: UITextField, context: Context) {
        if uiView.text != text {
            uiView.text = text
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(text: $text, digitCount: digitCount)
    }

    class Coordinator: NSObject, UITextFieldDelegate {
        @Binding var text: String
        let digitCount: Int

        init(text: Binding<String>, digitCount: Int) {
            _text = text
            self.digitCount = digitCount
        }

        @objc func textChanged(_ textField: UITextField) {
            let filtered = (textField.text ?? "").filter { $0.isNumber }
            let clamped = String(filtered.prefix(digitCount))
            if textField.text != clamped {
                textField.text = clamped
            }
            text = clamped
        }

        func textField(_ textField: UITextField, shouldChangeCharactersIn range: NSRange, replacementString string: String) -> Bool {
            let currentText = textField.text ?? ""
            guard let stringRange = Range(range, in: currentText) else { return false }
            let updatedText = currentText.replacingCharacters(in: stringRange, with: string)
            let filtered = updatedText.filter { $0.isNumber }
            return filtered.count <= digitCount
        }
    }
}

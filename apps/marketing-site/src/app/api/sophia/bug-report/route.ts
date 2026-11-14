// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import { NextRequest, NextResponse } from 'next/server'

const BUG_REPORT_EMAIL = process.env.BUG_REPORT_EMAIL || ''

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const {
      title,
      description,
      stepsToReproduce,
      expectedBehavior,
      actualBehavior,
      severity,
      environment,
      qorId,
      contactEmail,
    } = body

    // Validate required fields
    if (!title || !description) {
      return NextResponse.json(
        { error: 'Title and description are required' },
        { status: 400 }
      )
    }

    // Format bug report
    const bugReport = `
Bug Report Submitted via Sophia AI

Title: ${title}
Severity: ${severity || 'Medium'}
QOR ID: ${qorId || 'Not provided'}
Contact Email: ${contactEmail || 'Not provided'}

Description:
${description}

Steps to Reproduce:
${stepsToReproduce || 'Not provided'}

Expected Behavior:
${expectedBehavior || 'Not provided'}

Actual Behavior:
${actualBehavior || 'Not provided'}

Environment:
${environment || 'Not provided'}

Submitted: ${new Date().toISOString()}
    `.trim()

    // TODO: Implement email sending or database storage in production
    // Example: await sendEmail(BUG_REPORT_EMAIL, 'Bug Report from Sophia', bugReport)

    return NextResponse.json({
      success: true,
      message: 'Bug report submitted successfully. Thank you for helping improve Demiurge-Cloud!',
      reportId: `BUG-${Date.now()}`,
    })
  } catch (error: any) {
    console.error('Bug report error:', error)
    return NextResponse.json(
      { error: error.message || 'Failed to submit bug report' },
      { status: 500 }
    )
  }
}
